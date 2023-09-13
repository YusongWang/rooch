/// This module keeps a global wall clock that stores the current Unix time in microseconds.
/// It interacts with the other modules in the following ways:
/// * genesis: to initialize the timestamp
/// * L1 block: update the timestamp via L1s block header timestamp
/// * TickTransaction: update the timestamp via the time offset in the TickTransaction(TODO)
module rooch_framework::timestamp {
   
    use std::error;
    use moveos_std::account_storage;
    use moveos_std::storage_context::StorageContext;

    friend rooch_framework::genesis;
    friend rooch_framework::ethereum_light_client;

    /// A singleton resource holding the current Unix time in microseconds
    struct CurrentTimeMicroseconds has key {
        microseconds: u64,
    }

    /// Conversion factor between seconds and microseconds
    const MICRO_CONVERSION_FACTOR: u64 = 1000000;

    /// An invalid timestamp was provided
    const ErrorInvalidTimestamp: u64 = 1;

    public(friend) fun genesis_init(ctx: &mut StorageContext, genesis_account: &signer, initial_time_microseconds: u64) {
        let current_time = CurrentTimeMicroseconds { microseconds: initial_time_microseconds };
        account_storage::global_move_to(ctx, genesis_account, current_time);
    }

    /// Updates the wall clock time, if the new time is smaller than the current time, aborts.
    public(friend) fun update_global_time(ctx: &mut StorageContext,timestamp: u64) {
        let global_timer = account_storage::global_borrow_mut<CurrentTimeMicroseconds>(ctx, @rooch_framework);
        let now = global_timer.microseconds;
        assert!(now < timestamp, error::invalid_argument(ErrorInvalidTimestamp));
        global_timer.microseconds = timestamp;
    }

    /// Tries to update the wall clock time, if the new time is smaller than the current time, ignores the update, and returns false.
    public(friend) fun try_update_global_time(ctx: &mut StorageContext, timestamp: u64) : bool {
        let global_timer = account_storage::global_borrow_mut<CurrentTimeMicroseconds>(ctx, @rooch_framework);
        let now = global_timer.microseconds;
        if(now < timestamp) {
            global_timer.microseconds = timestamp;
            true
        }else{
            false
        }
    }

    #[view]
    /// Gets the current time in microseconds.
    public fun now_microseconds(ctx: &StorageContext): u64 {
        account_storage::global_borrow<CurrentTimeMicroseconds>(ctx, @rooch_framework).microseconds
    }

    #[view]
    /// Gets the current time in seconds.
    public fun now_seconds(ctx: &StorageContext): u64 {
        now_microseconds(ctx) / MICRO_CONVERSION_FACTOR
    }

    public fun seconds_to_microseconds(seconds: u64): u64 {
        seconds * MICRO_CONVERSION_FACTOR
    }

    #[test_only]
    public fun update_global_time_for_test(ctx: &mut StorageContext, timestamp_microsecs: u64){
        let global_timer = account_storage::global_borrow_mut<CurrentTimeMicroseconds>(ctx, @rooch_framework);
        let now = global_timer.microseconds;
        assert!(now < timestamp_microsecs, error::invalid_argument(ErrorInvalidTimestamp));
        global_timer.microseconds = timestamp_microsecs;
    }

    #[test_only]
    public fun update_global_time_for_test_secs(ctx: &mut StorageContext, timestamp_seconds: u64) {
        update_global_time_for_test(ctx, timestamp_seconds * MICRO_CONVERSION_FACTOR);
    }

    #[test_only]
    public fun fast_forward_seconds(ctx: &mut StorageContext, timestamp_seconds: u64) {
        let now_seconds = now_seconds(ctx);
        update_global_time_for_test_secs(ctx, now_seconds + timestamp_seconds);
    }
}
