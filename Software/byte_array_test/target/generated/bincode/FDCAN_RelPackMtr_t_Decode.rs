impl < __Context > :: bincode :: Decode < __Context > for FDCAN_RelPackMtr_t
{
    fn decode < __D : :: bincode :: de :: Decoder < Context = __Context > >
    (decoder : & mut __D) ->core :: result :: Result < Self, :: bincode ::
    error :: DecodeError >
    {
        core :: result :: Result ::
        Ok(Self
        {
            mtr_volt : :: bincode :: Decode :: decode(decoder) ?, mtr_curr :
            :: bincode :: Decode :: decode(decoder) ?,
        })
    }
} impl < '__de, __Context > :: bincode :: BorrowDecode < '__de, __Context >
for FDCAN_RelPackMtr_t
{
    fn borrow_decode < __D : :: bincode :: de :: BorrowDecoder < '__de,
    Context = __Context > > (decoder : & mut __D) ->core :: result :: Result <
    Self, :: bincode :: error :: DecodeError >
    {
        core :: result :: Result ::
        Ok(Self
        {
            mtr_volt : :: bincode :: BorrowDecode ::< '_, __Context >::
            borrow_decode(decoder) ?, mtr_curr : :: bincode :: BorrowDecode
            ::< '_, __Context >:: borrow_decode(decoder) ?,
        })
    }
}