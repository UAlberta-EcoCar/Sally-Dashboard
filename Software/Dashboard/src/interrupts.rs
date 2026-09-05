use embassy_stm32::{
    bind_interrupts, can, dma, exti, interrupt,
    peripherals::{self, FDCAN2},
};
bind_interrupts!(
    pub struct Irqs {
        FDCAN2_IT0 => can::IT0InterruptHandler<FDCAN2>;
        FDCAN2_IT1 => can::IT1InterruptHandler<FDCAN2>;
        DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
        DMA1_CHANNEL2 => dma::InterruptHandler<peripherals::DMA1_CH2>;
        DMA2_CHANNEL1 => dma::InterruptHandler<peripherals::DMA2_CH1>;
        EXTI3 => exti::InterruptHandler<interrupt::typelevel::EXTI3>;
        EXTI4 => exti::InterruptHandler<interrupt::typelevel::EXTI4>;
    }
);
