# Transaction Engine

## Description

The project was built using Rust 2021.

There are two main modules: `types` and `engine`.

### How it works

In the async `main` function a multi producer - single consumer channel is created. A task is spawnned in order to read from CSV file and send the transactions to the channel, while another task is spawnned to read from the channel and add those transactions to the engine which will process the data.

This approach was chosen because it'd be advantageous while processing huge files. As the channel has a max buffer, the data will be processed by the engine task thus not loading the complete file in the memory. Disk IO bottleneck is also tackled that way due to async.

This async + channel approach also allows the program to be easily extended, allowing multiple tasks to add transactions to the channel. Those tasks could be implemented to read input data from the network, tcp streams, fifo, etc, and all of them would run concurrently.

### Assumptions

- Input data is always valid. Which means, there won't be an initial deposit to a client X with `tx_id` Y, then a dispute to a client Z with the same `tx_id` Y. Since `tx_id` is globally unique it's not checking for correctness of the input data.
- Still on the input data, amounts are always positive. If a negative is found it'll affect the correctness of the output.
- The history of transactions amount is kept in memory. In a production code it should go to a database (persistent or memory). As this is not the case here a huge amount of transactions can impact the memory usage. However, a hashmap containing only the amount and a regular/under dispute flag is stored to consume less memory and these amounts are cleaned up from when a dispute is solved.
- Output order of accounts is unsorted. The output is not being sorted to improve performance.

### Usage

The defaul logging level is none. To increase the logging level just use the `RUST_LOG` variable.

If the variable `RUST_LOG` is set, it'll produce data to the standard output affecting the output file. Make sure to unset it when needed.

```
RUST_LOG=debug cargo run --release -- example.csv
RUST_LOG=trace cargo run --release -- example.csv
```
