#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CCW {
    CounterClockwise = 1,
    Clockwise = -1,
    OnLineBack = 2,
    OnLineFront = -2,
    // Endpoint-inclusive
    OnSegment = 0,
}
