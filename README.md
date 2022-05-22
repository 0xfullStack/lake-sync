# lake-sync
[![ Continuous Integration](https://github.com/cryptoless/lake-sync/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/cryptoless/lake-sync/actions/workflows/ci.yml)


Cryptoless lake-sync is a tool for polling and subscribing LP info quickly from EVM compatible.

Cryptoless lake-sync is an open source Rust implementation that event sources the Ethereum blockchain to deterministically update a data store that can be queried via lake-scan rest api.


## Quick Start

### Prerequisites

To build and run this project you need to have the following installed on your system:

- Rust (latest stable) – [How to install Rust](https://www.rust-lang.org/en-US/install.html)
  - Note that `rustfmt`, which is part of the default Rust installation, is a build-time requirement.
- PostgreSQL – [PostgreSQL Downloads](https://www.postgresql.org/download/)

For Ethereum network data, you can either run your own Ethereum node or use an Ethereum node provider of your choice.

**Minimum Hardware Requirements:**

- To build lake-sync with `cargo`, 8GB RAM are required.

## Project Layout

- `abi` — Used to store contract abi json files.
- `db` — Generate connection to PostgreSQL using Diesel orm framework.
- `dex` — core polling and syncing logic, divided into data assembler and data subscriber.

## Roadmap

🔨 = In Progress

🛠 = Feature complete. Additional testing required.

✅ = Feature complete


| Feature |  Status |
| ------- |  :------: |
| **Protocols** |    |
| Uniswap-v2 | ✅ |
| **Test** |     |
| Unit Test | 🛠 |


## License

Copyright &copy; 2021-2022 Cryptoless, Inc. and contributors.

Cryptoless lake-sync is dual-licensed under the [MIT license](LICENSE-MIT) and the [Apache License, Version 2.0](LICENSE-APACHE).

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either expressed or implied. See the License for the specific language governing permissions and limitations under the License.
