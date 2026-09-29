const WEI_PER_GWEI: u128 = 1_000_000_000;
const WEI_PER_ETH: u128 = 1_000_000_000_000_000_000;

enum Unit {
    Wei,
    Gwei,
    Eth,
}

impl Unit {
    // Converts a value that is ALWAYS represented in Wei
    // into the target unit.
    //
    // Example:
    // 1_000_000_000 Wei -> 1 Gwei
    // 1_000_000_000_000_000_000 Wei -> 1 ETH
    fn convert(&self, value_in_wei: u128) -> u128 {
        match self {
            // The value is already in Wei.
            Unit::Wei => value_in_wei,

            // Convert Wei to Gwei.
            Unit::Gwei => value_in_wei / WEI_PER_GWEI,

            // Convert Wei to ETH.
            Unit::Eth => value_in_wei / WEI_PER_ETH,
        }
    }
}

fn parse_unit(s: &str) -> Result<Unit, String> {
    match s {
        "wei" => Ok(Unit::Wei),
        "gwei" => Ok(Unit::Gwei),
        "eth" => Ok(Unit::Eth),

        _ => Err(format!(
            "unrecognised unit '{}'. Valid units are: wei, gwei, eth",
            s
        )),
    }
}

fn parse_amount(s: &str, unit: &Unit) -> Result<u128, String> {
    match unit {
        // Wei is already our internal unit,
        // so we can parse the number directly.
        Unit::Wei => {
            s.parse::<u128>()
                .map_err(|_| format!("invalid amount: {}", s))
        }

        // Gwei needs to be converted to Wei.
        //
        // Example:
        // "1" Gwei
        //     ↓
        // 1 * 1_000_000_000
        //     ↓
        // 1_000_000_000 Wei
        Unit::Gwei => {
            let amount = s
                .parse::<u128>()
                .map_err(|_| format!("invalid amount: {}", s))?;

            amount
                .checked_mul(WEI_PER_GWEI)
                .ok_or_else(|| format!("amount too large: {}", s))
        }

        // ETH can contain decimal values.
        //
        // Example:
        //
        // "1.5" ETH
        //
        // becomes:
        //
        // 1 ETH
        // +
        // 0.5 ETH
        //
        // and is finally represented as:
        //
        // 1_500_000_000_000_000_000 Wei
        Unit::Eth => {
            const DECIMALS: usize = 18;

            // Split the ETH amount around the decimal point.
            //
            // "1.5" -> ["1", "5"]
            let parts: Vec<&str> = s.split('.').collect();

            // An amount such as "1.5.2" is invalid.
            if parts.len() > 2 {
                return Err(format!("invalid amount: {}", s));
            }

            // Parse the whole-number part.
            //
            // "1" -> 1
            let integer_part = parts[0]
                .parse::<u128>()
                .map_err(|_| format!("invalid amount: {}", s))?;

            // Get the fractional part if one exists.
            //
            // "1.5" -> "5"
            //
            // We then pad it with zeros on the RIGHT
            // until it contains 18 digits.
            //
            // "5"
            //   ->
            // "500000000000000000"
            let fractional_part = if parts.len() == 2 {
                let fraction = parts[1];

                // Ethereum uses 18 decimal places because
                // 1 ETH = 10^18 Wei.
                if fraction.len() > DECIMALS {
                    return Err(format!(
                        "too many decimal places: {}",
                        fraction.len()
                    ));
                }

                format!("{:0<18}", fraction)
            } else {
                // No decimal part was provided.
                //
                // Example:
                // "2" -> "000000000000000000"
                "0".repeat(DECIMALS)
            };

            // Convert the padded fractional part to a number.
            //
            // "500000000000000000"
            // ->
            // 500_000_000_000_000_000
            let fractional_wei = fractional_part
                .parse::<u128>()
                .map_err(|_| format!("invalid amount: {}", s))?;

            // Convert the integer ETH portion to Wei.
            //
            // 1 ETH
            // ->
            // 1_000_000_000_000_000_000 Wei
            let integer_wei = integer_part
                .checked_mul(WEI_PER_ETH)
                .ok_or_else(|| format!("amount too large: {}", s))?;

            // Add the integer and fractional Wei portions.
            //
            // 1 ETH
            // + 0.5 ETH
            // =
            // 1.5 ETH
            //
            // =
            // 1_500_000_000_000_000_000 Wei
            integer_wei
                .checked_add(fractional_wei)
                .ok_or_else(|| format!("amount too large: {}", s))
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    // We expect exactly 4 arguments:
    //
    // args[0] = program name
    // args[1] = amount
    // args[2] = from-unit
    // args[3] = to-unit
    //
    // Example:
    //
    // cargo run -- 1.5 eth gwei

    if args.len() != 4 {
        return Err(
            "expected 3 arguments: <amount> <from-unit> <to-unit>"
                .to_string(),
        );
    }

    // Parse the unit we are converting FROM.
    //
    // We need this because parse_amount needs to know
    // how to interpret the user's input.
    let from_unit = parse_unit(&args[2])?;

    // Parse the unit we are converting TO.
    let to_unit = parse_unit(&args[3])?;

    // Parse the amount.
    //
    // IMPORTANT:
    //
    // parse_amount ALWAYS returns Wei.
    //
    // So:
    //
    // "1 wei"  -> 1 Wei
    // "1 gwei" -> 1_000_000_000 Wei
    // "1.5 eth" -> 1_500_000_000_000_000_000 Wei
    let amount_in_wei = parse_amount(&args[1], &from_unit)?;

    // Convert the Wei value into the target unit.
    //
    // Notice that we no longer pass from_unit.
    //
    // convert() only needs to know the target unit because
    // amount_in_wei is ALWAYS Wei.
    let result = to_unit.convert(amount_in_wei);

    // Display the result.
    println!(
        "{} {} = {} {}",
        args[1], args[2], result, args[3]
    );

    Ok(())
}

fn main() {
    // Collect all command-line arguments into a Vec<String>.
    let args: Vec<String> = std::env::args().collect();

    // Run the program.
    //
    // If run() succeeds:
    //
    // Ok(())
    //
    // nothing happens here.
    //
    // If run() fails:
    //
    // Err(e)
    //
    // print the error and exit with status code 1.
    if let Err(e) = run(&args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unit_valid_inputs() {
        assert!(matches!(parse_unit("wei"), Ok(Unit::Wei)));
        assert!(matches!(parse_unit("gwei"), Ok(Unit::Gwei)));
        assert!(matches!(parse_unit("eth"), Ok(Unit::Eth)));
    }

    #[test]
    fn test_parse_unit_invalid_input() {
        let result = parse_unit("bitcoin");

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_amount_wei() {
        let result = parse_amount("1000000000", &Unit::Wei);

        assert_eq!(result.unwrap(), 1_000_000_000);
    }

    #[test]
    fn test_parse_amount_gwei() {
        let result = parse_amount("1", &Unit::Gwei);

        assert_eq!(result.unwrap(), 1_000_000_000);
    }

    #[test]
    fn test_parse_amount_eth_whole_number() {
        let result = parse_amount("1", &Unit::Eth);

        assert_eq!(
            result.unwrap(),
            1_000_000_000_000_000_000
        );
    }

    #[test]
    fn test_parse_amount_eth_decimal() {
        let result = parse_amount("1.5", &Unit::Eth);

        assert_eq!(
            result.unwrap(),
            1_500_000_000_000_000_000
        );
    }

    #[test]
    fn test_convert_wei_to_gwei() {
        let value_in_wei = 1_000_000_000;

        let result = Unit::Gwei.convert(value_in_wei);

        assert_eq!(result, 1);
    }

    #[test]
    fn test_convert_wei_to_eth() {
        let value_in_wei = 1_000_000_000_000_000_000;

        let result = Unit::Eth.convert(value_in_wei);

        assert_eq!(result, 1);
    }
}