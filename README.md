# Ethereum Unit Converter

A small Rust command-line tool for converting between Wei, Gwei, and ETH.

## Usage

Build and run the program with Cargo:

```bash
cargo run -- <amount> <from-unit> <to-unit>
```

For example:

```bash
cargo run -- 1000000000 wei gwei
```

Output:

```text
1000000000 wei = 1 gwei
```

Another example:

```bash
cargo run -- 1 gwei wei
```

Output:

```text
1 gwei = 1000000000 wei
```

ETH values can include up to 18 decimal places:

```bash
cargo run -- 1.5 eth wei
```

Output:

```text
1.5 eth = 1500000000000000000 wei
```

The supported units are:

```text
wei
gwei
eth
```

Invalid units and amounts return an error instead of producing a result.

## What I learned from Rust enums

I could have represented the units as strings such as `"wei"`, `"gwei"`, and `"eth"`. An enum gives the program a fixed set of valid values instead.

```rust
enum Unit {
    Wei,
    Gwei,
    Eth,
}
```

That means functions such as `parse_unit` can turn user input into one of these known variants. The rest of the program can then use `match` to handle each case.

It also means I cannot accidentally create a `Unit` containing an arbitrary string. The type itself describes the valid choices.

## Financial arithmetic

The converter uses `u128` rather than floating-point numbers.

This matters because ETH has a fixed relationship with Wei:

```text
1 ETH = 1,000,000,000,000,000,000 Wei
```

The smallest unit is Wei, so decimal ETH input is converted into an exact integer number of Wei.

For example:

```text
1.5 ETH
= 1 ETH + 0.5 ETH
= 1,500,000,000,000,000,000 Wei
```

Floating-point numbers are not a good fit for this kind of arithmetic because they cannot represent every decimal value exactly. Small rounding errors can therefore appear when values are converted or calculated.

Using integers lets the program work with exact Wei values.

## The key design decision

The main design decision was to use Wei as the internal representation for every amount.

`parse_amount` converts the user's input into Wei first:

```text
1000000000 wei → 1,000,000,000 Wei

1 gwei → 1,000,000,000 Wei

1.5 eth → 1,500,000,000,000,000,000 Wei
```

After that, `convert` only receives a Wei value and converts it to the requested unit.

This removed the need for `convert` to know the original unit. It only needs the target unit.

For example:

```rust
Unit::Wei => value_in_wei,
Unit::Gwei => value_in_wei / WEI_PER_GWEI,
Unit::Eth => value_in_wei / WEI_PER_ETH,
```

This also prevented the double-conversion bug I found earlier. If `parse_amount` has already converted `1.5 ETH` into Wei, `convert` should not multiply it by `WEI_PER_ETH` again.

The internal flow is therefore:

```text
user input
    ↓
parse_unit
    ↓
parse_amount
    ↓
Wei
    ↓
convert
    ↓
requested output unit
```

## Tests

The project includes tests for unit parsing, amount parsing, ETH decimal handling, and conversions.

Run them with:

```bash
cargo test
```
