#![allow(dead_code)] // temporary

use std::{collections::HashMap, default::Default};

use sodiumoxide::{
    self,
    crypto::{box_, sealedbox, sign},
    randombytes::randombytes,
};

use super::{utils::is_eligible, PetError};

/// Tasks of a participant.
enum Task {
    Sum,
    Update,
    None,
}

/// A participant in the PET protocol layer.
pub struct Participant {
    // credentials
    encr_pk: box_::PublicKey,
    encr_sk: box_::SecretKey,
    sign_pk: sign::PublicKey,
    sign_sk: sign::SecretKey,
    ephm_pk: box_::PublicKey,
    ephm_sk: box_::SecretKey,
    certificate: Vec<u8>,
    signature_sum: sign::Signature,
    signature_update: sign::Signature,

    // other
    task: Task,
}

impl Participant {
    pub fn new() -> Result<Self, PetError> {
        // crucial: init must be called before anything else in this module
        sodiumoxide::init()
            .and(Ok(Default::default()))
            .or(Err(PetError::InvalidMessage))
    }

    /// Compute the "sum" and "update" signatures.
    pub fn compute_signatures(&mut self, round_seed: &[u8]) {
        self.signature_sum = sign::sign_detached(&[round_seed, b"sum"].concat(), &self.sign_sk);
        self.signature_update =
            sign::sign_detached(&[round_seed, b"update"].concat(), &self.sign_sk);
    }

    /// Check eligibility for a task.
    pub fn check_task(&mut self, round_sum: f64, round_update: f64) -> Result<(), PetError> {
        if is_eligible(&self.signature_sum, round_sum).ok_or(PetError::InvalidMessage)? {
            self.task = Task::Sum;
            Ok(())
        } else if is_eligible(&self.signature_update, round_update)
            .ok_or(PetError::InvalidMessage)?
        {
            self.task = Task::Update;
            Ok(())
        } else {
            self.task = Task::None;
            Ok(())
        }
    }
}

impl Default for Participant {
    fn default() -> Self {
        let (encr_pk, encr_sk) = box_::gen_keypair();
        let (sign_pk, sign_sk) = sign::gen_keypair();
        let ephm_pk = box_::PublicKey([0_u8; box_::PUBLICKEYBYTES]);
        let ephm_sk = box_::SecretKey([0_u8; box_::SECRETKEYBYTES]);
        let certificate: Vec<u8> = Vec::new();
        let signature_sum = sign::Signature([0_u8; sign::SIGNATUREBYTES]);
        let signature_update = sign::Signature([0_u8; sign::SIGNATUREBYTES]);
        let task = Task::None;
        Self {
            encr_pk,
            encr_sk,
            sign_pk,
            sign_sk,
            ephm_pk,
            ephm_sk,
            certificate,
            signature_sum,
            signature_update,
            task,
        }
    }
}

// Message egress with buffers:
//
// encr_pk -┐
// sign_pk -┤
//          └-> SealedBoxBuffer
//               └-> SealedBox -------┐
// certificate ------┐                |
// signature_sum ----┤                |
// signature_update -┤                |
// ephm_pk ----------┤                |
//                   └-> SumBoxBuffer |
//                        └-> SumBox -┤
//                                    └-> MessageBuffer
//                                         └-> SumMessage
//
// encr_pk -┐
// sign_pk -┤
//          └-> SealedBoxBuffer
//               └-> SealedBox ----------┐
// certificate ------┐                   |
// signature_sum ----┤                   |
// signature_update -┤                   |
// model_url---------┤                   |
// dict_seed---------┤                   |
//                   └-> UpdateBoxBuffer |
//                        └-> UpdateBox -┤
//                                       └-> MessageBuffer
//                                            └-> UpdateMessage
//
// encr_pk -┐
// sign_pk -┤
//          └-> SealedBoxBuffer
//               └-> SealedBox --------┐
// certificate ------┐                 |
// signature_sum ----┤                 |
// signature_update -┤                 |
// mask_url ---------┤                 |
//                   └-> Sum2BoxBuffer |
//                        └-> Sum2Box -┤
//                                     └-> MessageBuffer
//                                          └-> Sum2Message

/// Buffer and wrap the asymmetrically encrypted part of a "sum/update/sum2" message.
struct SealedBoxBuffer<'tag, 'encr_key, 'sign_key>(&'tag [u8], &'encr_key [u8], &'sign_key [u8]);

impl<'tag, 'encr_key, 'sign_key> SealedBoxBuffer<'tag, 'encr_key, 'sign_key> {
    fn new(encr_pk: &'encr_key box_::PublicKey, sign_pk: &'sign_key sign::PublicKey) -> Self {
        Self(
            b"round",       // 5 bytes
            &encr_pk.0[..], // 32 bytes
            &sign_pk.0[..], // 32 bytes
        ) // 69 bytes in total
    }

    fn seal(&self, coord_encr_pk: &box_::PublicKey) -> Vec<u8> {
        sealedbox::seal(&[self.0, self.1, self.2].concat(), coord_encr_pk) // 48 + 69 bytes, 117 bytes in total
    }
}

/// Buffer and wrap the symmetrically encrypted part of a "sum" message.
struct SumBoxBuffer<'tag, 'cert, 'sign_, 'ephm_key>(
    &'tag [u8],
    &'cert [u8],
    &'sign_ [u8],
    &'sign_ [u8],
    &'ephm_key [u8],
);

impl<'tag, 'cert, 'sign_, 'ephm_key> SumBoxBuffer<'tag, 'cert, 'sign_, 'ephm_key> {
    fn new(
        certificate: &'cert [u8],
        signature_sum: &'sign_ sign::Signature,
        signature_update: &'sign_ sign::Signature,
        ephm_pk: &'ephm_key box_::PublicKey,
    ) -> Self {
        Self(
            b"sum",                  // 3 bytes
            certificate,             // 0 bytes (dummy)
            &signature_sum.0[..],    // 64 bytes
            &signature_update.0[..], // 64 bytes
            &ephm_pk.0[..],          // 32 bytes
        ) // 163 bytes in total
    }

    fn seal(&self, coord_encr_pk: &box_::PublicKey, part_encr_sk: &box_::SecretKey) -> Vec<u8> {
        let nonce = box_::gen_nonce(); // 24 bytes
        let sumbox = box_::seal(
            &[self.0, self.1, self.2, self.3, self.4].concat(),
            &nonce,
            coord_encr_pk,
            part_encr_sk,
        ); // 16 + 163 bytes
        [nonce.0.to_vec(), sumbox].concat() // 203 bytes in total
    }
}

/// Buffer and wrap the symmetrically encrypted part of an "update" message.
struct UpdateBoxBuffer<'tag, 'cert, 'sign_, 'url, 'dict>(
    &'tag [u8],
    &'cert [u8],
    &'sign_ [u8],
    &'sign_ [u8],
    &'url [u8],
    &'dict [u8],
);

impl<'tag, 'cert, 'sign_, 'url, 'dict> UpdateBoxBuffer<'tag, 'cert, 'sign_, 'url, 'dict> {
    fn new(
        certificate: &'cert [u8],
        signature_sum: &'sign_ sign::Signature,
        signature_update: &'sign_ sign::Signature,
        model_url: &'url [u8],
        dict_seed: &'dict [u8],
    ) -> Self {
        Self(
            b"update",               // 6 bytes
            certificate,             // 0 bytes (dummy)
            &signature_sum.0[..],    // 64 bytes
            &signature_update.0[..], // 64 bytes
            model_url,               // 32 bytes (dummy)
            dict_seed,               // 112 * dict_sum.len() bytes
        ) // 166 + 112 * dict_sum.len() bytes in total
    }

    fn seal(&self, coord_encr_pk: &box_::PublicKey, part_encr_sk: &box_::SecretKey) -> Vec<u8> {
        let nonce = box_::gen_nonce(); // 24 bytes
        let updatebox = box_::seal(
            &[self.0, self.1, self.2, self.3, self.4, self.5].concat(),
            &nonce,
            coord_encr_pk,
            part_encr_sk,
        ); // 16 + 166 + 112 * dict_sum.len() bytes
        [nonce.0.to_vec(), updatebox].concat() // 206 + 112 * dict_sum.len() bytes in total
    }
}

/// Buffer and wrap the symmetrically encrypted part of a "sum2" message.
struct Sum2BoxBuffer<'tag, 'cert, 'sign_, 'url>(
    &'tag [u8],
    &'cert [u8],
    &'sign_ [u8],
    &'sign_ [u8],
    &'url [u8],
);

impl<'tag, 'cert, 'sign_, 'url> Sum2BoxBuffer<'tag, 'cert, 'sign_, 'url> {
    fn new(
        certificate: &'cert [u8],
        signature_sum: &'sign_ sign::Signature,
        signature_update: &'sign_ sign::Signature,
        mask_url: &'url [u8],
    ) -> Self {
        Self(
            &b"sum2"[..],            // 4 bytes
            certificate,             // 0 bytes (dummy)
            &signature_sum.0[..],    // 64 bytes
            &signature_update.0[..], // 64 bytes
            mask_url,                // 32 bytes (dummy)
        ) // 164 bytes in total
    }

    fn seal(&self, coord_encr_pk: &box_::PublicKey, part_encr_sk: &box_::SecretKey) -> Vec<u8> {
        let nonce = box_::gen_nonce(); // 24 bytes
        let sum2box = box_::seal(
            &[self.0, self.1, self.2, self.3, self.4].concat(),
            &nonce,
            coord_encr_pk,
            part_encr_sk,
        ); // 16 + 164 bytes
        [nonce.0.to_vec(), sum2box].concat() // 204 bytes in total
    }
}

/// Buffer and wrap an encrypted "sum/update/sum2" message.
struct MessageBuffer<'sbox, 'box___>(&'sbox [u8], &'box___ [u8]);

impl<'sbox, 'box___> MessageBuffer<'sbox, 'box___> {
    fn new(sealedbox: &'sbox [u8], box__: &'box___ [u8]) -> Self {
        Self(sealedbox, box__)
    }

    fn seal(&self) -> Vec<u8> {
        [self.0, self.1].concat()
    }
}

/// Compose and encrypt a "sum" message. Get an ephemeral asymmetric key pair.
pub struct SumMessage {
    message: Vec<u8>,
    part_ephm_pk: box_::PublicKey,
    part_ephm_sk: box_::SecretKey,
}

impl SumMessage {
    pub fn compose(part: &Participant, coord_encr_pk: &box_::PublicKey) -> Self {
        // generate ephemeral key pair
        let (part_ephm_pk, part_ephm_sk) = box_::gen_keypair();

        // encrypt message parts
        let sbox = SealedBoxBuffer::new(&part.encr_pk, &part.sign_pk).seal(coord_encr_pk);
        let sumbox = SumBoxBuffer::new(
            &part.certificate,
            &part.signature_sum,
            &part.signature_update,
            &part_ephm_pk,
        )
        .seal(coord_encr_pk, &part.encr_sk);
        let message = MessageBuffer::new(&sbox, &sumbox).seal();

        Self {
            message,
            part_ephm_pk,
            part_ephm_sk,
        }
    }
}

/// Compose and encrypt an "update" message. Get a seed of a local model mask.
pub struct UpdateMessage {
    message: Vec<u8>,
    mask_seed: Vec<u8>,
}

impl UpdateMessage {
    pub fn compose(
        part: &Participant,
        coord_encr_pk: &box_::PublicKey,
        dict_sum: &HashMap<box_::PublicKey, box_::PublicKey>,
    ) -> Self {
        // mask the local model
        let mask_seed = randombytes(32_usize);
        let model_url = randombytes(32_usize); // dummy

        // create dictionary of encrypted masking seeds
        let mut dict_seed: Vec<u8> = Vec::new();
        for (sum_encr_pk, sum_ephm_pk) in dict_sum.iter() {
            dict_seed.extend(sum_encr_pk.0.to_vec()); // 32 bytes
            dict_seed.extend(sealedbox::seal(&mask_seed, sum_ephm_pk)); // 48 + 32 bytes
        } // 112 * dict_sum.len() bytes in total

        // encrypt message parts
        let sbox = SealedBoxBuffer::new(&part.encr_pk, &part.sign_pk).seal(coord_encr_pk);
        let updatebox = UpdateBoxBuffer::new(
            &part.certificate,
            &part.signature_sum,
            &part.signature_update,
            &model_url,
            &dict_seed,
        )
        .seal(coord_encr_pk, &part.encr_sk);
        let message = MessageBuffer::new(&sbox, &updatebox).seal();

        Self { message, mask_seed }
    }
}

/// Compose and encrypt a "sum" message. Get an url of a global mask.
pub struct Sum2Message {
    message: Vec<u8>,
    mask_url: Vec<u8>,
}

impl Sum2Message {
    pub fn compose(
        part: &Participant,
        coord_encr_pk: &box_::PublicKey,
        dict_seed: &HashMap<box_::PublicKey, HashMap<box_::PublicKey, Vec<u8>>>,
    ) -> Result<Self, PetError> {
        // compute global mask
        let mut seeds: Vec<Vec<u8>> = Vec::new();
        for seed in dict_seed
            .get(&part.encr_pk)
            .ok_or(PetError::InvalidMessage)?
            .values()
        {
            seeds.append(&mut vec![sealedbox::open(
                seed,
                &part.ephm_pk,
                &part.ephm_sk,
            )
            .or(Err(PetError::InvalidMessage))?]);
        }
        let mask_url = randombytes(32_usize); // dummy

        // encrypt message parts
        let sbox = SealedBoxBuffer::new(&part.encr_pk, &part.sign_pk).seal(coord_encr_pk);
        let sum2box = Sum2BoxBuffer::new(
            &part.certificate,
            &part.signature_sum,
            &part.signature_update,
            &mask_url,
        )
        .seal(coord_encr_pk, &part.encr_sk);
        let message = MessageBuffer::new(&sbox, &sum2box).seal();

        Ok(Self { message, mask_url })
    }
}
