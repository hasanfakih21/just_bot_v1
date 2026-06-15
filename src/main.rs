fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    justbot::tools::uci::input_loop(args.join(" "));
}
