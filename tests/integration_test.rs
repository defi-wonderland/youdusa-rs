use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

fn load_test_file(filename: &str) -> File {
    let mut test_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_file_path.push("tests");
    test_file_path.push(filename);

    File::open(test_file_path).unwrap_or_else(|_| panic!("Failed to open test file: {}", filename))
}

#[test]
fn test_empty_input() {
    let input = Cursor::new("your test input here");
    let mut output = Vec::new(); // Vec implements Write, not String :/

    youdusa::process_input(Box::new(input), &mut output).unwrap();

    // Convert output to string for assertions
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.is_empty());
}

#[test]
fn test_simple_log() {
    let input = load_test_file("test_log_simple.txt");
    let mut output = Vec::new();

    youdusa::process_input(Box::new(input), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();

    assert!(!output_str.is_empty());
    assert_eq!(
        output_str,
"    function test_prop_anyoneCanIncreaseFundInAPool() public {
        vm.roll(10429);
        vm.warp(19960);
        vm.prank(address(0x0000000000000000000000000000000000050000));
        this.prop_alloOwnerCanAlwaysChangePercentFee{ value: 123 }(15056796);

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_tryThisNow{ value: 12345678901234567890 }(13441534537036760751763869415731560796441041418, 334437, (123,''), (0x123, 69),'');

    }\n\n");
}

#[test]
fn test_multiple_log() {
    let input = load_test_file("test_log_multiple.txt");
    let mut output = Vec::new();

    youdusa::process_input(Box::new(input), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();

    assert!(!output_str.is_empty());
    assert_eq!(
        output_str,
"    function test_prop_anyoneCanIncreaseFundInAPool() public {
        vm.roll(10429);
        vm.warp(19960);
        vm.prank(address(0x0000000000000000000000000000000000050000));
        this.prop_alloOwnerCanAlwaysChangePercentFee(15056796);

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

    }

    function test_prop_anyoneCanIncreaseFundInAPool2() public {
        vm.roll(10429);
        vm.warp(19960);
        vm.prank(address(0x0000000000000000000000000000000000050000));
        this.prop_alloOwnerCanAlwaysChangePercentFee(15056796);

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(23, 334437, (1, 2),'');

    }\n\n");
}

#[test]
fn test_long_seq() {
    let input = load_test_file("test_log_long_seq.txt");
    let mut output = Vec::new();

    youdusa::process_input(Box::new(input), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();

    assert!(!output_str.is_empty());
    assert_eq!(
        output_str,
"    function test_prop_anyoneCanIncreaseFundInAPool() public {
        vm.roll(10429);
        vm.warp(19960);
        vm.prank(address(0x0000000000000000000000000000000000050000));
        this.prop_alloOwnerCanAlwaysChangePercentFee(15056796);

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(address(0x0000000000000000000000000000000000070000));
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

    }\n\n");
}
