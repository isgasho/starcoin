// Utility for converting a Move value to its binary representation in LCS (Libra Canonical
// Serialization). LCS is the binary encoding for Move resources and other non-module values
// published on-chain. See https://github.com/libra/libra/tree/master/common/lcs for more
// details on LCS (TODO: link to spec once we have one)

address 0x1 {
module LCS {
    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }
    // Return the binary representation of `v` in LCS (Libra Canonical Serialization) format
    native public fun to_bytes<MoveValue>(v: &MoveValue): vector<u8>;

    // Return the address of key bytes
    native public fun to_address(key_bytes: vector<u8>): address;
    // ------------------------------------------------------------------------
    // Specification
    // ------------------------------------------------------------------------

    spec module {
        native define serialize<MoveValue>(v: &MoveValue): vector<u8>;
    }
}
}
