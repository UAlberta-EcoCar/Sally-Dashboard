impl :: bincode :: Encode for FDCAN_RelPackMtr_t
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) ->core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    {
        :: bincode :: Encode :: encode(&self.mtr_volt, encoder) ?; :: bincode
        :: Encode :: encode(&self.mtr_curr, encoder) ?; core :: result ::
        Result :: Ok(())
    }
}