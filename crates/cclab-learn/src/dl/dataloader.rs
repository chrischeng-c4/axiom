//! DataLoader for batching and shuffling training data.

use super::tensor::Tensor;

/// A simple DataLoader that yields `(input, target)` batches.
///
/// Iterates over a dataset in mini-batches, with optional shuffling.
pub struct DataLoader {
    /// Input data: `[num_samples, ...]`
    pub inputs: Tensor,
    /// Target data: `[num_samples, ...]`
    pub targets: Tensor,
    pub batch_size: usize,
    pub shuffle: bool,
    seed: u64,
}

impl DataLoader {
    /// Create a new DataLoader.
    ///
    /// `inputs` and `targets` must have the same first dimension (number of samples).
    pub fn new(
        inputs: Tensor,
        targets: Tensor,
        batch_size: usize,
        shuffle: bool,
        seed: u64,
    ) -> Self {
        assert!(
            !inputs.shape.is_empty() && !targets.shape.is_empty(),
            "inputs and targets must be non-empty"
        );
        assert_eq!(
            inputs.shape[0], targets.shape[0],
            "inputs and targets must have the same number of samples"
        );
        assert!(batch_size > 0, "batch_size must be > 0");
        Self {
            inputs,
            targets,
            batch_size,
            shuffle,
            seed,
        }
    }

    /// Number of samples in the dataset.
    pub fn len(&self) -> usize {
        self.inputs.shape[0]
    }

    /// Whether the dataset is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Number of batches per epoch.
    pub fn num_batches(&self) -> usize {
        (self.len() + self.batch_size - 1) / self.batch_size
    }

    /// Create an iterator over batches for one epoch.
    pub fn iter(&self) -> DataLoaderIter<'_> {
        let n = self.len();
        let indices: Vec<usize> = if self.shuffle {
            let mut idx: Vec<usize> = (0..n).collect();
            // Fisher-Yates shuffle with LCG
            let mut rng = self.seed;
            for i in (1..n).rev() {
                rng = rng
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let j = (rng as usize) % (i + 1);
                idx.swap(i, j);
            }
            idx
        } else {
            (0..n).collect()
        };

        DataLoaderIter {
            loader: self,
            indices,
            pos: 0,
        }
    }
}

/// Iterator over mini-batches from a `DataLoader`.
pub struct DataLoaderIter<'a> {
    loader: &'a DataLoader,
    indices: Vec<usize>,
    pos: usize,
}

impl<'a> Iterator for DataLoaderIter<'a> {
    type Item = (Tensor, Tensor);

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.loader.len();
        if self.pos >= n {
            return None;
        }

        let end = (self.pos + self.loader.batch_size).min(n);
        let batch_indices = &self.indices[self.pos..end];
        let actual_batch_size = batch_indices.len();
        self.pos = end;

        // Compute per-sample sizes
        let input_sample_size = self.loader.inputs.numel() / n;
        let target_sample_size = self.loader.targets.numel() / n;

        // Gather input batch
        let mut input_data = Vec::with_capacity(actual_batch_size * input_sample_size);
        for &idx in batch_indices {
            let start = idx * input_sample_size;
            let end = start + input_sample_size;
            input_data.extend_from_slice(&self.loader.inputs.data[start..end]);
        }
        let mut input_shape = self.loader.inputs.shape.clone();
        input_shape[0] = actual_batch_size;

        // Gather target batch
        let mut target_data = Vec::with_capacity(actual_batch_size * target_sample_size);
        for &idx in batch_indices {
            let start = idx * target_sample_size;
            let end = start + target_sample_size;
            target_data.extend_from_slice(&self.loader.targets.data[start..end]);
        }
        let mut target_shape = self.loader.targets.shape.clone();
        target_shape[0] = actual_batch_size;

        Some((
            Tensor::new(input_data, input_shape),
            Tensor::new(target_data, target_shape),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataloader_basic() {
        let inputs = Tensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0], vec![4, 2]);
        let targets = Tensor::new(vec![0.0, 1.0, 0.0, 1.0], vec![4, 1]);
        let loader = DataLoader::new(inputs, targets, 2, false, 42);

        assert_eq!(loader.len(), 4);
        assert_eq!(loader.num_batches(), 2);

        let batches: Vec<_> = loader.iter().collect();
        assert_eq!(batches.len(), 2);

        // First batch: samples 0, 1
        assert_eq!(batches[0].0.shape, vec![2, 2]);
        assert_eq!(batches[0].0.data, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(batches[0].1.shape, vec![2, 1]);
        assert_eq!(batches[0].1.data, vec![0.0, 1.0]);

        // Second batch: samples 2, 3
        assert_eq!(batches[1].0.shape, vec![2, 2]);
        assert_eq!(batches[1].0.data, vec![5.0, 6.0, 7.0, 8.0]);
    }

    #[test]
    fn test_dataloader_partial_batch() {
        let inputs = Tensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![3, 2]);
        let targets = Tensor::new(vec![0.0, 1.0, 2.0], vec![3, 1]);
        let loader = DataLoader::new(inputs, targets, 2, false, 42);

        assert_eq!(loader.num_batches(), 2);
        let batches: Vec<_> = loader.iter().collect();
        assert_eq!(batches.len(), 2);
        // Last batch has 1 sample
        assert_eq!(batches[1].0.shape, vec![1, 2]);
        assert_eq!(batches[1].1.shape, vec![1, 1]);
    }

    #[test]
    fn test_dataloader_shuffle() {
        let inputs = Tensor::new((0..20).map(|i| i as f64).collect(), vec![10, 2]);
        let targets = Tensor::new((0..10).map(|i| i as f64).collect(), vec![10, 1]);
        let loader = DataLoader::new(inputs, targets, 10, true, 42);

        let batches: Vec<_> = loader.iter().collect();
        assert_eq!(batches.len(), 1);

        // Shuffled targets should be a permutation of 0..10
        let mut sorted_targets = batches[0].1.data.clone();
        sorted_targets.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let expected: Vec<f64> = (0..10).map(|i| i as f64).collect();
        assert_eq!(sorted_targets, expected);
    }

    #[test]
    fn test_dataloader_multiple_epochs() {
        let inputs = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]);
        let targets = Tensor::new(vec![0.0, 1.0], vec![2, 1]);
        let loader = DataLoader::new(inputs, targets, 1, false, 42);

        // Should be able to iterate multiple times
        for _ in 0..3 {
            let batches: Vec<_> = loader.iter().collect();
            assert_eq!(batches.len(), 2);
        }
    }

    #[test]
    fn test_dataloader_is_empty() {
        let inputs = Tensor::new(vec![1.0], vec![1, 1]);
        let targets = Tensor::new(vec![0.0], vec![1, 1]);
        let loader = DataLoader::new(inputs, targets, 1, false, 42);
        assert!(!loader.is_empty());
    }
}
