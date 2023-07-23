# resback
resback is a part of respec.team platform and this provides backend features.\
The name comes from "res(pec team's) back(end)".

## Requirements
Make sure these dependencies are already installed.
- Git
- Rust (version 1.6.0 or higher)

## Getting Started
1. Clone this repository.
``` shell
git clone https://github.com/respec-team/resback
```

2. Build an executable using Cargo.
``` shell
cargo build
```

3. Create `.env` file
``` shell
# Example of applying production environment settings
ln -s $(pwd)/.env.prod $(pwd)/.env
```

4. Run the executable.
``` shell
cargo run
```

## Author
[Qoo](https://github.com/qoor) (akck0918@gmail.com)

## Copyright
Copyright 2023. [The resback authors](AUTHORS) all rights reserved.
