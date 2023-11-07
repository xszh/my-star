use anyhow::Result;
use rubato::SincFixedIn;

pub fn resample<T: rubato::Sample>(
  source: f64,
  target: f64,
  chunk_size: usize,
) -> Result<SincFixedIn<T>> {
  use rubato::{SincInterpolationParameters, SincInterpolationType, WindowFunction};
  let params = SincInterpolationParameters {
    sinc_len: 256,
    f_cutoff: 0.95,
    interpolation: SincInterpolationType::Linear,
    oversampling_factor: 256,
    window: WindowFunction::BlackmanHarris2,
  };
  let resampler: SincFixedIn<T> =
    SincFixedIn::<T>::new(target / source, 1.0, params, chunk_size, 1)?;

  Ok(resampler)
}
