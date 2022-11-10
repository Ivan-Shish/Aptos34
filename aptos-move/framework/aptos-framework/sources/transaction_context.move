module aptos_framework::transaction_context {
    /// Return the script hash of the current entry function.
    public native fun get_script_hash(): vector<u8>;

    public native fun get_bucket(): u64;
}
