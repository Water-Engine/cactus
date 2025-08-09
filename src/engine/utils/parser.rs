
pub fn position(args: Vec<String>) -> Result<(), String> {
    let is_uci_str = args.contains(&"startpos".to_string());
    let is_fen_str = args.contains(&"fen".to_string());
    if is_uci_str && is_fen_str {
        return Err("Invalid position command: expected either 'startpos' or 'fen', received both".into());
    }

    if is_uci_str {

    } else if is_fen_str {

    } else {
        return Err("Invalid position command: expected either 'startpos' or 'fen'".into());
    }
    Ok(())
}