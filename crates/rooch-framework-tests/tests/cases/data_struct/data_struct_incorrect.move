//# init --addresses creator=0x42

//# publish
module creator::test1 {
    

    struct DisallowedStruct has copy,drop {
        f_u8: u8,
    }

    #[data_struct(T)]
    public fun f1<T: copy+drop>(_data: T) {
    }

    public fun f2() {
        let disallowed_struct = DisallowedStruct {
            f_u8: 123,
        };
        f1(disallowed_struct);
    }
}

