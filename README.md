# Mortgage simulator written in Rust.

This is a small project that can simulate mortgage payments for different payment schemes. It is written in Rust and provides a command-line interface for users to input their mortgage details and export the amortization schedule of the monthly payments to a csv file. The following payment schemes are supported:
- `fixed capital`: monthly payment consists of a fixed capital payment (= principal / total number of months) and interest on outstanding capital.
- `fixed mensualities`: fixed monthly payments containing a variable mix of capital and interest over the period of the mortgage.
- `variable linear capital`: monthly payment consists of a variable capital payment that increases/decreases with a fixed amount $\delta$ each period and interest on outstanding capital. $\delta$ is determined automatically based on the desired initial payment, the initial interest rate and the length of the mortgage.

This is an educational project for me to learn Rust.
