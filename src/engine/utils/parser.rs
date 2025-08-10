pub fn position(args: Vec<String>) -> Result<Vec<String>, String> {
    let is_uci_str = args.contains(&"startpos".to_string());
    let is_fen_str = args.contains(&"fen".to_string());
    if is_uci_str && is_fen_str {
        return Err(
            "Invalid position command: expected either 'startpos' or 'fen', received both".into(),
        );
    }

    let mut moves = Vec::new();
    let mut args_iter = args.iter();
    if args_iter.find(|&s| s == "moves").is_none() {
        return Err("Invalid position command: failed to find move label in args".into());
    }
    args_iter.for_each(|next_move| moves.push(next_move.to_string()));

    if is_uci_str {
    } else if is_fen_str {
    } else {
        return Err("Invalid position command: expected either 'startpos' or 'fen'".into());
    }
    Ok(moves)
}
