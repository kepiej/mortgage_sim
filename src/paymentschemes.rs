use crate::mortgage::{Mortgage, month_interest_rate};
use csv::Writer;
use std::fmt;
use std::str;

#[derive(Debug, PartialEq)]
pub enum PaymentScheme {
    FixedCapital,
    FixedMensualities,
    VariableLinearCapital(f64),
}

impl str::FromStr for PaymentScheme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        match parts.as_slice() {
            ["FixedCapital"] | ["VasteKapitaalaflossing"] => Ok(PaymentScheme::FixedCapital),
            ["FixedMensualities"] | ["VasteMensualiteiten"] => Ok(PaymentScheme::FixedMensualities),
            ["VariableLinearCapital", init_pay]
            | ["VariabeleLineaireKapitaalaflossing", init_pay] => {
                match init_pay.trim().parse::<f64>() {
                    Ok(value) => Ok(PaymentScheme::VariableLinearCapital(value)),
                    Err(_) => Err(format!("Invalid initial payment: '{}'", init_pay)),
                }
            }
            _ => Err("Invalid input".to_string()),
        }
    }
}

impl fmt::Display for PaymentScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FixedCapital => write!(f, "{}", "FixedCapital"),
            Self::FixedMensualities => write!(f, "{}", "FixedMensualities"),
            Self::VariableLinearCapital(init_pay) => {
                write!(f, "VariableLinearCapital {}", init_pay)
            }
        }
    }
}

pub struct MortgagePayments {
    mortgage: Mortgage,
    payments: Vec<f64>,
    capitalpaid: Vec<f64>,
}

impl MortgagePayments {
    pub fn new(m: Mortgage, p: PaymentScheme) -> Self {
        let (payments, capitalpaid): (Vec<f64>, Vec<f64>) = match p {
            PaymentScheme::FixedCapital => {
                fixed_capital_payments(m.capital, m.nperiods, &m.year_interest_rate)
            }
            PaymentScheme::FixedMensualities => {
                fixed_mensualities(m.capital, m.nperiods, &m.year_interest_rate)
            }
            PaymentScheme::VariableLinearCapital(init_pay) => variable_linear_capital_payments(
                m.capital,
                m.nperiods,
                &m.year_interest_rate,
                init_pay,
            ),
        };

        return Self {
            mortgage: m,
            payments: payments,
            capitalpaid: capitalpaid,
        };
    }

    pub fn payments(&self) -> Vec<f64> {
        return self.payments.to_vec();
    }

    pub fn capital_paid(&self) -> Vec<f64> {
        return self.capitalpaid.to_vec();
    }

    pub fn total_repaid(&self) -> f64 {
        return self.payments().iter().sum();
    }

    pub fn to_csv(&self, filename: String) {
        let payments: Vec<f64> = self.payments();
        let capitalpaid: Vec<f64> = self.capital_paid();
        let saldo: Vec<f64> = capitalpaid
            .iter()
            .scan(0.0, |acc, e| {
                *acc += e;
                Some(self.mortgage.capital - *acc + e)
            })
            .collect();

        let interests: Vec<f64> = payments
            .iter()
            .zip(capitalpaid.iter())
            .map(|(a, b)| a - b)
            .collect();

        //TODO: Fix file path!
        let mut wtr = Writer::from_path(filename).expect("Bestand kon niet gemaakt worden!");
        wtr.serialize((
            "month",
            "payment",
            "saldo",
            "interest_rate",
            "capital_paid",
            "interests_paid",
        ))
        .expect("Failed");

        let monthly_interest_rate: Vec<f64> = self.mortgage.monthly_interest_rate();
        for i in 0usize..self.mortgage.nperiods() as usize {
            wtr.serialize((
                i + 1,
                payments[i],
                saldo[i],
                monthly_interest_rate[i],
                capitalpaid[i],
                interests[i],
            ))
            .expect("Failed");
        }
        wtr.flush().expect("Flush failed");
    }
}

fn fixed_capital_payments(
    capital: f64,
    nperiods: i64,
    year_interest_rate: &[f64],
) -> (Vec<f64>, Vec<f64>) {
    let mut payments: Vec<f64> = Vec::new();
    let fixedcap: f64 = capital / nperiods as f64;

    let m_interest_rate: Vec<f64> = month_interest_rate(&year_interest_rate);
    let mut saldo: f64;
    let mut mens: f64;
    for period in 1..nperiods + 1 {
        saldo = capital - (period - 1) as f64 * fixedcap;
        mens = fixedcap + m_interest_rate[(period - 1) as usize] * saldo;
        payments.push(mens);
    }
    let capitalpaid: Vec<f64> = vec![fixedcap; nperiods as usize];

    return (payments, capitalpaid);
}

fn fixed_mensualities(
    capital: f64,
    nperiods: i64,
    year_interest_rate: &[f64],
) -> (Vec<f64>, Vec<f64>) {
    let mut payments: Vec<f64> = Vec::new();
    let m_interest_rate: Vec<f64> = month_interest_rate(&year_interest_rate);
    for period in 1..nperiods + 1 {
        let mens: f64 = (capital
            * m_interest_rate[(period - 1) as usize]
            * (1.0 + m_interest_rate[(period - 1) as usize]).powf(nperiods as f64))
            / ((1.0 + m_interest_rate[(period - 1) as usize]).powf(nperiods as f64) - 1.0);
        payments.push(mens);
    }
    return (payments, vec![f64::NAN; nperiods as usize]);
}

fn variable_linear_capital_payments(
    capital: f64,
    nperiods: i64,
    year_interest_rate: &[f64],
    initial_payment: f64,
) -> (Vec<f64>, Vec<f64>) {
    let m_interest_rate: Vec<f64> = month_interest_rate(&year_interest_rate);
    let delta: f64 = 2.0
        * (capital - (nperiods as f64) * (initial_payment - (m_interest_rate[0] * capital)))
        / ((nperiods - 1) * nperiods) as f64;
    println!("{delta}");
    let mut xt: Vec<f64> = Vec::new();
    xt.push(initial_payment - (m_interest_rate[0] * capital));
    let mut payments: Vec<f64> = Vec::new();
    let mut mens: f64;
    let mut saldo: f64 = capital;
    for period in 1..nperiods + 1 {
        mens = xt[(period - 1) as usize] + m_interest_rate[(period - 1) as usize] * saldo;
        payments.push(mens);
        saldo = capital - xt[..period as usize].iter().sum::<f64>();
        xt.push(xt[(period - 1) as usize] + delta);
    }
    xt.pop();
    return (payments, xt);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_paymentscheme() {
        let mut pay: PaymentScheme = "FixedCapital".parse::<PaymentScheme>().unwrap();
        assert_eq!(PaymentScheme::FixedCapital, pay);

        pay = "FixedMensualities".parse::<PaymentScheme>().unwrap();
        assert_eq!(PaymentScheme::FixedMensualities, pay);

        pay = "VariableLinearCapital 500.0"
            .parse::<PaymentScheme>()
            .unwrap();
        assert_eq!(PaymentScheme::VariableLinearCapital(500.0), pay);
    }

    #[test]
    fn test_fixed_capital_payments() {
        assert_eq!(
            fixed_capital_payments(92000.0, 1, &[1.8 / 100.0; 1]).0[0],
            92000.0 * (1.0 + month_interest_rate(&[1.8 / 100.0; 1])[0])
        )
    }

    #[test]
    fn test_fixed_mensualities() {
        assert!(
            fixed_mensualities(92000.0, 1, &[1.8 / 100.0; 1]).0[0]
                - 92000.0 * (1.0 + month_interest_rate(&[1.8 / 100.0; 1])[0])
                <= 1e-6
        )
    }

    #[test]
    fn test_variable_linear_capital_payments() {
        //TODO
        assert_eq!(true, false);
    }
}
