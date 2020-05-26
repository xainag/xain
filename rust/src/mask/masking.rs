use rand::SeedableRng;
use std::iter::{self, Iterator};

use num::{
    bigint::{BigInt, BigUint, ToBigInt},
    clamp,
    rational::Ratio,
};
use rand_chacha::ChaCha20Rng;

use crate::{
    crypto::generate_integer,
    mask::{MaskConfig, MaskObject, MaskSeed, Model},
};

use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum UnmaskingError {
    #[error("there is no model to unmask")]
    NoModel,

    #[error("too many models were aggregated for the current unmasking configuration")]
    TooManyModels,

    #[error("the masked model is incompatible with the mask used for unmasking")]
    MaskMismatch,

    #[error("the mask is invalid")]
    InvalidMask,
}

#[derive(Debug, Error)]
pub enum AggregationError {
    #[error("the model to aggregate is invalid")]
    InvalidModel,

    #[error("too many models were aggregated for the current unmasking configuration")]
    TooManyModels,

    #[error("the model to aggregate is incompatible with the current aggregated model")]
    ModelMismatch,
}

#[derive(Debug)]
pub struct Aggregation {
    nb_models: usize,
    object: MaskObject,
}

impl From<MaskObject> for Aggregation {
    fn from(object: MaskObject) -> Self {
        Self {
            nb_models: 1,
            object,
        }
    }
}

impl Into<MaskObject> for Aggregation {
    fn into(self) -> MaskObject {
        self.object
    }
}

impl Aggregation {
    pub fn new(config: MaskConfig) -> Self {
        Self {
            nb_models: 0,
            object: MaskObject::new(config, vec![]),
        }
    }

    pub fn config(&self) -> MaskConfig {
        self.object.config
    }

    pub fn validate_unmasking(&self, mask: &MaskObject) -> Result<(), UnmaskingError> {
        // We cannot perform unmasking without at least one real model
        if self.nb_models == 0 {
            return Err(UnmaskingError::NoModel);
        }

        if self.nb_models > self.object.config.model_type.nb_models_max() {
            return Err(UnmaskingError::TooManyModels);
        }

        if self.object.config != mask.config || self.object.data.len() != mask.data.len() {
            return Err(UnmaskingError::MaskMismatch);
        }

        if !mask.is_valid() {
            return Err(UnmaskingError::InvalidMask);
        }

        Ok(())
    }

    pub fn unmask(mut self, mask: MaskObject) -> Model {
        let scaled_add_shift = self.object.config.add_shift() * BigInt::from(self.nb_models);
        let exp_shift = self.object.config.exp_shift();
        let order = self.object.config.order();
        self.object
            .data
            .drain(..)
            .zip(mask.data.into_iter())
            .map(|(masked_weight, mask)| {
                // PANIC_SAFE: The substraction panics if it
                // underflows, which can only happen if:
                //
                //     mask > self.object.config.order()
                //
                // If the mask is valid, we are guaranteed that this
                // cannot happen. Thus this method may panic only if
                // given an invalid mask.
                let n = (masked_weight + &order - mask) % &order;

                // UNWRAP_SAFE: to_bigint never fails for BigUint
                let ratio = Ratio::<BigInt>::from(n.to_bigint().unwrap());

                ratio / &exp_shift - &scaled_add_shift
            })
            .collect()
    }

    pub fn validate_aggregation(&self, object: &MaskObject) -> Result<(), AggregationError> {
        if self.object.config != object.config {
            return Err(AggregationError::ModelMismatch);
        }

        // If we have at least one object, make sure the object we're
        // trying to aggregate has the same length.
        if self.nb_models > 0 && (self.object.data.len() != object.data.len()) {
            return Err(AggregationError::ModelMismatch);
        }

        if self.nb_models == self.object.config.model_type.nb_models_max() {
            return Err(AggregationError::TooManyModels);
        }

        if !object.is_valid() {
            return Err(AggregationError::InvalidModel);
        }

        Ok(())
    }

    pub fn aggregate(&mut self, object: MaskObject) {
        if self.nb_models == 0 {
            self.object = object;
            self.nb_models = 1;
            return;
        }

        let order = self.object.config.order();
        for (i, j) in self.object.data.iter_mut().zip(object.data.into_iter()) {
            *i = (&*i + j) % &order
        }
        self.nb_models += 1;
    }
}

pub struct Masker {
    pub config: MaskConfig,
    pub seed: MaskSeed,
}

impl Masker {
    pub fn new(config: MaskConfig) -> Self {
        Self {
            config,
            seed: MaskSeed::generate(),
        }
    }

    pub fn with_seed(config: MaskConfig, seed: MaskSeed) -> Self {
        Self { config, seed }
    }
}

impl Masker {
    /// Mask the model wrt the mask configuration. Enforces bounds on the scalar and weights.
    ///
    /// The masking proceeds in the following steps:
    /// - clamp the scalar and the weights according to the mask configuration
    /// - shift the weights into the non-negative reals
    /// - shift the weights into the non-negative integers
    /// - shift the weights into the finite group
    /// - mask the weights with random elements from the finite group
    ///
    /// The random elements are derived from a seeded PRNG. Unmasking proceeds in reverse order. For
    /// more details see [the confluence page](https://xainag.atlassian.net/wiki/spaces/FP/pages/542408769/Masking).
    pub fn mask(self, scalar: f64, model: Model) -> (MaskSeed, MaskObject) {
        let random_ints = self.random_ints();

        let Self { seed, config } = self;

        let exp_shift = config.exp_shift();
        let add_shift = config.add_shift();
        let order = config.order();
        let higher_bound = &add_shift;
        let lower_bound = -&add_shift;
        let scalar = Ratio::<BigInt>::from_float(clamp(scalar, 0_f64, 1_f64)).unwrap();
        let masked_weights = model
            .into_iter()
            .zip(random_ints)
            .map(|(weight, rand_int)| {
                let scaled = &scalar * clamp(&weight, &lower_bound, higher_bound);
                // PANIC_SAFE: shifted weight is guaranteed to be non-negative
                let shifted = ((scaled + &add_shift) * &exp_shift)
                    .to_integer()
                    .to_biguint()
                    .unwrap();
                (shifted + rand_int) % &order
            })
            .collect();
        let masked_model = MaskObject::new(config, masked_weights);
        (seed, masked_model)
    }

    fn random_ints(&self) -> impl Iterator<Item = BigUint> {
        let order = self.config.order();
        let mut prng = ChaCha20Rng::from_seed(self.seed.as_array());

        iter::from_fn(move || Some(generate_integer(&mut prng, &order)))
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use num::traits::Signed;
    use rand::{
        distributions::{Distribution, Uniform},
        SeedableRng,
    };
    use rand_chacha::ChaCha20Rng;

    use super::*;
    use crate::mask::{
        config::{
            BoundType::{Bmax, B0, B2, B4, B6},
            DataType::{F32, F64, I32, I64},
            GroupType::{Integer, Power2, Prime},
            MaskConfig,
            ModelType::M3,
        },
        model::FromPrimitives,
    };

    /// Generate tests for masking and unmasking of a single model:
    /// - generate random weights from a uniform distribution with a seeded PRNG
    /// - create a model from the weights and mask it
    /// - check that all masked weights belong to the chosen finite group
    /// - unmask the masked model
    /// - check that all unmasked weights are equal to the original weights (up to a tolerance
    ///   determined by the masking configuration)
    ///
    /// The arguments to the macro are:
    /// - a suffix for the test name
    /// - the group type of the model (variants of `GroupType`)
    /// - the data type of the model (either primitives or variants of `DataType`)
    /// - an absolute bound for the weights (optional, choices: 1, 100, 10_000, 1_000_000)
    /// - the number of weights
    macro_rules! test_masking {
        ($suffix:ident, $group:ty, $data:ty, $bound:expr, $len:expr $(,)?) => {
            paste::item! {
                #[test]
                fn [<test_masking_ $suffix>]() {
                    // Step 1: Build the masking config
                    let config = MaskConfig {
                        group_type: $group,
                        data_type: paste::expr! { [<$data:upper>] },
                        bound_type: match $bound {
                            1 => B0,
                            100 => B2,
                            10_000 => B4,
                            1_000_000 => B6,
                            _ => Bmax,
                        },
                        model_type: M3,
                    };

                    // Step 2: Generate a random model
                    let bound = if $bound == 0 {
                        paste::expr! { [<$data:lower>]::MAX / (2 as [<$data:lower>]) }
                    } else {
                        paste::expr! { $bound as [<$data:lower>] }
                    };
                    let mut prng = ChaCha20Rng::from_seed(MaskSeed::generate().as_array());
                    let random_weights = Uniform::new_inclusive(-bound, bound)
                        .sample_iter(&mut prng)
                        .take($len as usize);
                    let model = Model::from_primitives(random_weights).unwrap();

                    // Step 3 (actual test):
                    // a. mask the model
                    // b. derive the mask corresponding to the seed used
                    // c. unmask the model and check it against the original one.
                    let (mask_seed, masked_model) = Masker::new(config.clone()).mask(1_f64, model.clone());
                    assert_eq!(masked_model.data.len(), model.len());
                    assert!(masked_model.is_valid());

                    let mask = mask_seed.derive_mask(model.len(), config);
                    let aggregation = Aggregation::from(masked_model);
                    let unmasked_model = aggregation.unmask(mask);

                    let tolerance = Ratio::from_integer(config.exp_shift()).recip();
                    assert!(
                        model.iter()
                            .zip(unmasked_model.iter())
                            .all(|(weight, unmasked_weight)| {
                                (weight - unmasked_weight).abs() <= tolerance
                            })
                    );
                }
            }
        };
        ($suffix:ident, $group:ty, $data:ty, $len:expr $(,)?) => {
            test_masking!($suffix, $group, $data, 0, $len);
        };
    }

    test_masking!(int_f32_b0, Integer, f32, 1, 10);
    test_masking!(int_f32_b2, Integer, f32, 100, 10);
    test_masking!(int_f32_b4, Integer, f32, 10_000, 10);
    test_masking!(int_f32_b6, Integer, f32, 1_000_000, 10);
    test_masking!(int_f32_bmax, Integer, f32, 10);

    test_masking!(prime_f32_b0, Prime, f32, 1, 10);
    test_masking!(prime_f32_b2, Prime, f32, 100, 10);
    test_masking!(prime_f32_b4, Prime, f32, 10_000, 10);
    test_masking!(prime_f32_b6, Prime, f32, 1_000_000, 10);
    test_masking!(prime_f32_bmax, Prime, f32, 10);

    test_masking!(pow_f32_b0, Power2, f32, 1, 10);
    test_masking!(pow_f32_b2, Power2, f32, 100, 10);
    test_masking!(pow_f32_b4, Power2, f32, 10_000, 10);
    test_masking!(pow_f32_b6, Power2, f32, 1_000_000, 10);
    test_masking!(pow_f32_bmax, Power2, f32, 10);

    test_masking!(int_f64_b0, Integer, f64, 1, 10);
    test_masking!(int_f64_b2, Integer, f64, 100, 10);
    test_masking!(int_f64_b4, Integer, f64, 10_000, 10);
    test_masking!(int_f64_b6, Integer, f64, 1_000_000, 10);
    test_masking!(int_f64_bmax, Integer, f64, 10);

    test_masking!(prime_f64_b0, Prime, f64, 1, 10);
    test_masking!(prime_f64_b2, Prime, f64, 100, 10);
    test_masking!(prime_f64_b4, Prime, f64, 10_000, 10);
    test_masking!(prime_f64_b6, Prime, f64, 1_000_000, 10);
    test_masking!(prime_f64_bmax, Prime, f64, 10);

    test_masking!(pow_f64_b0, Power2, f64, 1, 10);
    test_masking!(pow_f64_b2, Power2, f64, 100, 10);
    test_masking!(pow_f64_b4, Power2, f64, 10_000, 10);
    test_masking!(pow_f64_b6, Power2, f64, 1_000_000, 10);
    test_masking!(pow_f64_bmax, Power2, f64, 10);

    test_masking!(int_i32_b0, Integer, i32, 1, 10);
    test_masking!(int_i32_b2, Integer, i32, 100, 10);
    test_masking!(int_i32_b4, Integer, i32, 10_000, 10);
    test_masking!(int_i32_b6, Integer, i32, 1_000_000, 10);
    test_masking!(int_i32_bmax, Integer, i32, 10);

    test_masking!(prime_i32_b0, Prime, i32, 1, 10);
    test_masking!(prime_i32_b2, Prime, i32, 100, 10);
    test_masking!(prime_i32_b4, Prime, i32, 10_000, 10);
    test_masking!(prime_i32_b6, Prime, i32, 1_000_000, 10);
    test_masking!(prime_i32_bmax, Prime, i32, 10);

    test_masking!(pow_i32_b0, Power2, i32, 1, 10);
    test_masking!(pow_i32_b2, Power2, i32, 100, 10);
    test_masking!(pow_i32_b4, Power2, i32, 10_000, 10);
    test_masking!(pow_i32_b6, Power2, i32, 1_000_000, 10);
    test_masking!(pow_i32_bmax, Power2, i32, 10);

    test_masking!(int_i64_b0, Integer, i64, 1, 10);
    test_masking!(int_i64_b2, Integer, i64, 100, 10);
    test_masking!(int_i64_b4, Integer, i64, 10_000, 10);
    test_masking!(int_i64_b6, Integer, i64, 1_000_000, 10);
    test_masking!(int_i64_bmax, Integer, i64, 10);

    test_masking!(prime_i64_b0, Prime, i64, 1, 10);
    test_masking!(prime_i64_b2, Prime, i64, 100, 10);
    test_masking!(prime_i64_b4, Prime, i64, 10_000, 10);
    test_masking!(prime_i64_b6, Prime, i64, 1_000_000, 10);
    test_masking!(prime_i64_bmax, Prime, i64, 10);

    test_masking!(pow_i64_b0, Power2, i64, 1, 10);
    test_masking!(pow_i64_b2, Power2, i64, 100, 10);
    test_masking!(pow_i64_b4, Power2, i64, 10_000, 10);
    test_masking!(pow_i64_b6, Power2, i64, 1_000_000, 10);
    test_masking!(pow_i64_bmax, Power2, i64, 10);

    /// Generate tests for aggregation of multiple masked models:
    /// - generate random integers from a uniform distribution with a seeded PRNG
    /// - create a masked model from the integers and aggregate it to the aggregated masked models
    /// - check that all integers belong to the chosen finite group
    ///
    /// The arguments to the macro are:
    /// - a suffix for the test name
    /// - the group type of the model (variants of `GroupType`)
    /// - the data type of the model (variants of `DataType`)
    /// - the bound type of the model (variants of `BoundType`)
    /// - the number of integers per masked model
    /// - the number of masked models
    macro_rules! test_aggregation {
        ($suffix:ident, $group:ty, $data:ty, $bound:expr, $len:expr, $count:expr $(,)?) => {
            paste::item! {
                #[test]
                fn [<test_aggregation_ $suffix>]() {
                    // Step 1: Build the masking config
                    let config = MaskConfig {
                        group_type: $group,
                        data_type: $data,
                        bound_type: $bound,
                        model_type: M3,
                    };

                    // Step 2: generate random masked models
                    let mut prng = ChaCha20Rng::from_seed(MaskSeed::generate().as_array());
                    let mut masked_models = iter::repeat_with(move || {
                        let order = config.order();
                        let integers = iter::repeat_with(|| generate_integer(&mut prng, &order))
                            .take($len as usize)
                            .collect::<Vec<_>>();
                        MaskObject::new(config, integers)
                    });

                    // Step 3 (actual test):
                    // a. aggregate the masked models
                    // b. check the aggregated masked model
                    let mut aggregated_masked_model = Aggregation::new(config);
                    for nb in 1..$count as usize + 1 {
                        let masked_model = masked_models.next().unwrap();
                        assert!(
                            aggregated_masked_model.validate_aggregation(&masked_model).is_ok()
                        );
                        aggregated_masked_model.aggregate(masked_model);

                        assert_eq!(aggregated_masked_model.nb_models, nb);
                        assert_eq!(aggregated_masked_model.object.data.len(), $len as usize);
                        assert_eq!(aggregated_masked_model.object.config, config);
                        assert!(aggregated_masked_model.object.is_valid());
                    }
                }
            }
        };
    }

    test_aggregation!(int_f32_b0, Integer, F32, B0, 10, 5);
    test_aggregation!(int_f32_b2, Integer, F32, B2, 10, 5);
    test_aggregation!(int_f32_b4, Integer, F32, B4, 10, 5);
    test_aggregation!(int_f32_b6, Integer, F32, B6, 10, 5);
    test_aggregation!(int_f32_bmax, Integer, F32, Bmax, 10, 5);

    test_aggregation!(prime_f32_b0, Prime, F32, B0, 10, 5);
    test_aggregation!(prime_f32_b2, Prime, F32, B2, 10, 5);
    test_aggregation!(prime_f32_b4, Prime, F32, B4, 10, 5);
    test_aggregation!(prime_f32_b6, Prime, F32, B6, 10, 5);
    test_aggregation!(prime_f32_bmax, Prime, F32, Bmax, 10, 5);

    test_aggregation!(pow_f32_b0, Power2, F32, B0, 10, 5);
    test_aggregation!(pow_f32_b2, Power2, F32, B2, 10, 5);
    test_aggregation!(pow_f32_b4, Power2, F32, B4, 10, 5);
    test_aggregation!(pow_f32_b6, Power2, F32, B6, 10, 5);
    test_aggregation!(pow_f32_bmax, Power2, F32, Bmax, 10, 5);

    test_aggregation!(int_f64_b0, Integer, F64, B0, 10, 5);
    test_aggregation!(int_f64_b2, Integer, F64, B2, 10, 5);
    test_aggregation!(int_f64_b4, Integer, F64, B4, 10, 5);
    test_aggregation!(int_f64_b6, Integer, F64, B6, 10, 5);
    test_aggregation!(int_f64_bmax, Integer, F64, Bmax, 10, 5);

    test_aggregation!(prime_f64_b0, Prime, F64, B0, 10, 5);
    test_aggregation!(prime_f64_b2, Prime, F64, B2, 10, 5);
    test_aggregation!(prime_f64_b4, Prime, F64, B4, 10, 5);
    test_aggregation!(prime_f64_b6, Prime, F64, B6, 10, 5);
    test_aggregation!(prime_f64_bmax, Prime, F64, Bmax, 10, 5);

    test_aggregation!(pow_f64_b0, Power2, F64, B0, 10, 5);
    test_aggregation!(pow_f64_b2, Power2, F64, B2, 10, 5);
    test_aggregation!(pow_f64_b4, Power2, F64, B4, 10, 5);
    test_aggregation!(pow_f64_b6, Power2, F64, B6, 10, 5);
    test_aggregation!(pow_f64_bmax, Power2, F64, Bmax, 10, 5);

    test_aggregation!(int_i32_b0, Integer, I32, B0, 10, 5);
    test_aggregation!(int_i32_b2, Integer, I32, B2, 10, 5);
    test_aggregation!(int_i32_b4, Integer, I32, B4, 10, 5);
    test_aggregation!(int_i32_b6, Integer, I32, B6, 10, 5);
    test_aggregation!(int_i32_bmax, Integer, I32, Bmax, 10, 5);

    test_aggregation!(prime_i32_b0, Prime, I32, B0, 10, 5);
    test_aggregation!(prime_i32_b2, Prime, I32, B2, 10, 5);
    test_aggregation!(prime_i32_b4, Prime, I32, B4, 10, 5);
    test_aggregation!(prime_i32_b6, Prime, I32, B6, 10, 5);
    test_aggregation!(prime_i32_bmax, Prime, I32, Bmax, 10, 5);

    test_aggregation!(pow_i32_b0, Power2, I32, B0, 10, 5);
    test_aggregation!(pow_i32_b2, Power2, I32, B2, 10, 5);
    test_aggregation!(pow_i32_b4, Power2, I32, B4, 10, 5);
    test_aggregation!(pow_i32_b6, Power2, I32, B6, 10, 5);
    test_aggregation!(pow_i32_bmax, Power2, I32, Bmax, 10, 5);

    test_aggregation!(int_i64_b0, Integer, I64, B0, 10, 5);
    test_aggregation!(int_i64_b2, Integer, I64, B2, 10, 5);
    test_aggregation!(int_i64_b4, Integer, I64, B4, 10, 5);
    test_aggregation!(int_i64_b6, Integer, I64, B6, 10, 5);
    test_aggregation!(int_i64_bmax, Integer, I64, Bmax, 10, 5);

    test_aggregation!(prime_i64_b0, Prime, I64, B0, 10, 5);
    test_aggregation!(prime_i64_b2, Prime, I64, B2, 10, 5);
    test_aggregation!(prime_i64_b4, Prime, I64, B4, 10, 5);
    test_aggregation!(prime_i64_b6, Prime, I64, B6, 10, 5);
    test_aggregation!(prime_i64_bmax, Prime, I64, Bmax, 10, 5);

    test_aggregation!(pow_i64_b0, Power2, I64, B0, 10, 5);
    test_aggregation!(pow_i64_b2, Power2, I64, B2, 10, 5);
    test_aggregation!(pow_i64_b4, Power2, I64, B4, 10, 5);
    test_aggregation!(pow_i64_b6, Power2, I64, B6, 10, 5);
    test_aggregation!(pow_i64_bmax, Power2, I64, Bmax, 10, 5);

    /// Generate tests for masking, aggregation and unmasking of multiple models:
    /// - generate random weights from a uniform distribution with a seeded PRNG
    /// - create a model from the weights, mask and aggregate it to the aggregated masked models
    /// - derive a mask from the mask seed and aggregate it to the aggregated masks
    /// - unmask the aggregated masked model
    /// - check that all aggregated unmasked weights are equal to the averaged original weights (up
    ///   to a tolerance determined by the masking configuration)
    ///
    /// The arguments to the macro are:
    /// - a suffix for the test name
    /// - the group type of the model (variants of `GroupType`)
    /// - the data type of the model (either primitives or variants of `DataType`)
    /// - an absolute bound for the weights (optional, choices: 1, 100, 10_000, 1_000_000)
    /// - the number of weights per model
    /// - the number of models
    macro_rules! test_masking_and_aggregation {
        ($suffix:ident, $group:ty, $data:ty, $bound:expr, $len:expr, $count:expr $(,)?) => {
            paste::item! {
                #[test]
                fn [<test_masking_and_aggregation_ $suffix>]() {
                    // Step 1: Build the masking config
                    let config = MaskConfig {
                        group_type: $group,
                        data_type: paste::expr! { [<$data:upper>] },
                        bound_type: match $bound {
                            1 => B0,
                            100 => B2,
                            10_000 => B4,
                            1_000_000 => B6,
                            _ => Bmax,
                        },
                        model_type: M3,
                    };

                    // Step 2: Generate random models
                    let bound = if $bound == 0 {
                        paste::expr! { [<$data:lower>]::MAX / (2 as [<$data:lower>]) }
                    } else {
                        paste::expr! { $bound as [<$data:lower>] }
                    };
                    let mut prng = ChaCha20Rng::from_seed(MaskSeed::generate().as_array());
                    let mut models = iter::repeat_with(move || {
                        Model::from_primitives(
                            Uniform::new_inclusive(-bound, bound)
                                .sample_iter(&mut prng)
                                .take($len as usize)
                        )
                        .unwrap()
                    });

                    // Step 3 (actual test):
                    // a. average the model weights for later checks
                    // b. mask the model
                    // c. derive the mask corresponding to the seed used
                    // d. aggregate the masked model resp. mask
                    // e. repeat a-d, then unmask the model and check it against the averaged one
                    let mut averaged_model = Model::from_primitives(
                        iter::repeat(paste::expr! { 0 as [<$data:lower>] }).take($len as usize)
                    )
                    .unwrap();
                    let mut aggregated_masked_model = Aggregation::new(config);
                    let mut aggregated_mask = Aggregation::new(config);
                    let scalar = 1_f64 / ($count as f64);
                    let scalar_ratio = Ratio::from_float(scalar).unwrap();
                    for _ in 0..$count as usize {
                        let model = models.next().unwrap();
                        averaged_model
                            .iter_mut()
                            .zip(model.iter())
                            .for_each(|(averaged_weight, weight)| {
                                *averaged_weight += &scalar_ratio * weight;
                            });

                        let (mask_seed, masked_model) = Masker::new(config).mask(scalar, model);
                        let mask = mask_seed.derive_mask($len as usize, config);

                        assert!(
                            aggregated_masked_model.validate_aggregation(&masked_model).is_ok()
                        );
                        aggregated_masked_model.aggregate(masked_model);
                        assert!(aggregated_mask.validate_aggregation(&mask).is_ok());
                        aggregated_mask.aggregate(mask);
                    }

                    let unmasked_model = aggregated_masked_model.unmask(aggregated_mask.into());
                    let tolerance = Ratio::from_integer(BigInt::from($count as usize))
                        / Ratio::from_integer(config.exp_shift());
                    assert!(
                        averaged_model.iter()
                            .zip(unmasked_model.iter())
                            .all(|(averaged_weight, unmasked_weight)| {
                                (averaged_weight - unmasked_weight).abs() <= tolerance
                            })
                    );
                }
            }
        };
        ($suffix:ident, $group:ty, $data:ty, $len:expr, $count:expr $(,)?) => {
            test_masking_and_aggregation!($suffix, $group, $data, 0, $len, $count);
        };
    }

    test_masking_and_aggregation!(int_f32_b0, Integer, f32, 1, 10, 5);
    test_masking_and_aggregation!(int_f32_b2, Integer, f32, 100, 10, 5);
    test_masking_and_aggregation!(int_f32_b4, Integer, f32, 10_000, 10, 5);
    test_masking_and_aggregation!(int_f32_b6, Integer, f32, 1_000_000, 10, 5);
    test_masking_and_aggregation!(int_f32_bmax, Integer, f32, 10, 5);

    test_masking_and_aggregation!(prime_f32_b0, Prime, f32, 1, 10, 5);
    test_masking_and_aggregation!(prime_f32_b2, Prime, f32, 100, 10, 5);
    test_masking_and_aggregation!(prime_f32_b4, Prime, f32, 10_000, 10, 5);
    test_masking_and_aggregation!(prime_f32_b6, Prime, f32, 1_000_000, 10, 5);
    test_masking_and_aggregation!(prime_f32_bmax, Prime, f32, 10, 5);

    test_masking_and_aggregation!(pow_f32_b0, Power2, f32, 1, 10, 5);
    test_masking_and_aggregation!(pow_f32_b2, Power2, f32, 100, 10, 5);
    test_masking_and_aggregation!(pow_f32_b4, Power2, f32, 10_000, 10, 5);
    test_masking_and_aggregation!(pow_f32_b6, Power2, f32, 1_000_000, 10, 5);
    test_masking_and_aggregation!(pow_f32_bmax, Power2, f32, 10, 5);

    test_masking_and_aggregation!(int_f64_b0, Integer, f64, 1, 10, 5);
    test_masking_and_aggregation!(int_f64_b2, Integer, f64, 100, 10, 5);
    test_masking_and_aggregation!(int_f64_b4, Integer, f64, 10_000, 10, 5);
    test_masking_and_aggregation!(int_f64_b6, Integer, f64, 1_000_000, 10, 5);
    test_masking_and_aggregation!(int_f64_bmax, Integer, f64, 10, 5);

    test_masking_and_aggregation!(prime_f64_b0, Prime, f64, 1, 10, 5);
    test_masking_and_aggregation!(prime_f64_b2, Prime, f64, 100, 10, 5);
    test_masking_and_aggregation!(prime_f64_b4, Prime, f64, 10_000, 10, 5);
    test_masking_and_aggregation!(prime_f64_b6, Prime, f64, 1_000_000, 10, 5);
    test_masking_and_aggregation!(prime_f64_bmax, Prime, f64, 10, 5);

    test_masking_and_aggregation!(pow_f64_b0, Power2, f64, 1, 10, 5);
    test_masking_and_aggregation!(pow_f64_b2, Power2, f64, 100, 10, 5);
    test_masking_and_aggregation!(pow_f64_b4, Power2, f64, 10_000, 10, 5);
    test_masking_and_aggregation!(pow_f64_b6, Power2, f64, 1_000_000, 10, 5);
    test_masking_and_aggregation!(pow_f64_bmax, Power2, f64, 10, 5);

    test_masking_and_aggregation!(int_i32_b0, Integer, i32, 1, 10, 5);
    test_masking_and_aggregation!(int_i32_b2, Integer, i32, 100, 10, 5);
    test_masking_and_aggregation!(int_i32_b4, Integer, i32, 10_000, 10, 5);
    test_masking_and_aggregation!(int_i32_b6, Integer, i32, 1_000_000, 10, 5);
    test_masking_and_aggregation!(int_i32_bmax, Integer, i32, 10, 5);

    test_masking_and_aggregation!(prime_i32_b0, Prime, i32, 1, 10, 5);
    test_masking_and_aggregation!(prime_i32_b2, Prime, i32, 100, 10, 5);
    test_masking_and_aggregation!(prime_i32_b4, Prime, i32, 10_000, 10, 5);
    test_masking_and_aggregation!(prime_i32_b6, Prime, i32, 1_000_000, 10, 5);
    test_masking_and_aggregation!(prime_i32_bmax, Prime, i32, 10, 5);

    test_masking_and_aggregation!(pow_i32_b0, Power2, i32, 1, 10, 5);
    test_masking_and_aggregation!(pow_i32_b2, Power2, i32, 100, 10, 5);
    test_masking_and_aggregation!(pow_i32_b4, Power2, i32, 10_000, 10, 5);
    test_masking_and_aggregation!(pow_i32_b6, Power2, i32, 1_000_000, 10, 5);
    test_masking_and_aggregation!(pow_i32_bmax, Power2, i32, 10, 5);

    test_masking_and_aggregation!(int_i64_b0, Integer, i64, 1, 10, 5);
    test_masking_and_aggregation!(int_i64_b2, Integer, i64, 100, 10, 5);
    test_masking_and_aggregation!(int_i64_b4, Integer, i64, 10_000, 10, 5);
    test_masking_and_aggregation!(int_i64_b6, Integer, i64, 1_000_000, 10, 5);
    test_masking_and_aggregation!(int_i64_bmax, Integer, i64, 10, 5);

    test_masking_and_aggregation!(prime_i64_b0, Prime, i64, 1, 10, 5);
    test_masking_and_aggregation!(prime_i64_b2, Prime, i64, 100, 10, 5);
    test_masking_and_aggregation!(prime_i64_b4, Prime, i64, 10_000, 10, 5);
    test_masking_and_aggregation!(prime_i64_b6, Prime, i64, 1_000_000, 10, 5);
    test_masking_and_aggregation!(prime_i64_bmax, Prime, i64, 10, 5);

    test_masking_and_aggregation!(pow_i64_b0, Power2, i64, 1, 10, 5);
    test_masking_and_aggregation!(pow_i64_b2, Power2, i64, 100, 10, 5);
    test_masking_and_aggregation!(pow_i64_b4, Power2, i64, 10_000, 10, 5);
    test_masking_and_aggregation!(pow_i64_b6, Power2, i64, 1_000_000, 10, 5);
    test_masking_and_aggregation!(pow_i64_bmax, Power2, i64, 10, 5);
}
