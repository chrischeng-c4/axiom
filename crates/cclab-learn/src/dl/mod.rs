//! Deep learning module — minimal autograd + neural network layers.
//!
//! - **tensor**: Tensor with automatic differentiation
//! - **nn**: Linear, ReLU, Sigmoid, Sequential, MSELoss, CrossEntropyLoss
//! - **optim**: SGD, Adam, AdamW, AdaGrad optimizers
//! - **activations**: Tanh, GELU, Softmax, LeakyReLU
//! - **layers**: Conv2d, Conv1d, BatchNorm, Dropout, MaxPool2d, AvgPool2d, Flatten, Embedding
//! - **recurrent**: LSTM, GRU
//! - **attention**: MultiHeadAttention
//! - **serialization**: Save/load model weights in JSON and binary formats
//! - **dataloader**: DataLoader for batching/shuffling

mod activations;
mod attention;
mod dataloader;
mod layers;
mod nn;
mod optim;
mod recurrent;
pub mod serialization;
mod tensor;

pub use activations::{LeakyReLU, Softmax, Tanh, GELU};
pub use attention::MultiHeadAttention;
pub use dataloader::{DataLoader, DataLoaderIter};
pub use layers::{AvgPool2d, BatchNorm, Conv1d, Conv2d, Dropout, Embedding, Flatten, MaxPool2d};
pub use nn::{CrossEntropyLoss, Layer, Linear, MSELoss, ReLU, Sequential, Sigmoid};
pub use optim::{AdaGrad, Adam, AdamW, SGD};
pub use recurrent::{GRU, LSTM};
pub use serialization::{extract_weights, load_weights, ModelWeights, ParamEntry, SerError};
pub use tensor::{Tape, Tensor};
