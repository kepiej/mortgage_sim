use csv::Writer;
use serde::Serialize;
use std::io;
use std::path::Path;

use tabled::settings::{Concat, Panel};
use tabled::{Table, Tabled};

#[derive(Serialize, Debug, Tabled)]
pub struct MonthlyPayment {
    month: usize,
    yearly_interest_rate: f64,
    payment: f64,
    capital: f64,
    interest: f64,
    balance: f64,
}

impl MonthlyPayment {
    pub fn new(
        month: usize,
        yearly_interest_rate: f64,
        payment: f64,
        capital: Option<f64>,
        balance: Option<f64>,
    ) -> Self {
        let capital_init: f64 = match capital {
            Some(capital) => capital,
            None => f64::NAN,
        };

        return Self {
            month: month,
            yearly_interest_rate: yearly_interest_rate,
            payment: payment,
            capital: capital_init,
            interest: payment - capital_init,
            balance: match balance {
                Some(balance) => balance,
                None => f64::NAN,
            },
        };
    }
}

pub trait MortgagePayments {
    fn payments(&self) -> Vec<f64>;

    fn capital_paid(&self) -> Vec<f64>;

    fn interest_paid(&self) -> Vec<f64>;

    fn total_repaid(&self) -> f64;

    fn to_csv(&self, filename: &Path) -> io::Result<()>;
}

pub fn display(payments: &Vec<MonthlyPayment>) -> String {
    let mut head: Table = Table::new(payments[..5].iter());
    let tail: Table = Table::nohead(payments[payments.len() - 5..].iter())
        .with(Panel::header(":"))
        .to_owned();
    return format!("{}", head.with(Concat::vertical(tail)));
}

impl MortgagePayments for Vec<MonthlyPayment> {
    fn payments(&self) -> Vec<f64> {
        return self.iter().map(|x| x.payment).collect();
    }

    fn capital_paid(&self) -> Vec<f64> {
        return self.iter().map(|x| x.capital).collect();
    }

    fn interest_paid(&self) -> Vec<f64> {
        return self.iter().map(|x| x.interest).collect();
    }

    fn total_repaid(&self) -> f64 {
        return self.payments().iter().sum();
    }

    fn to_csv(&self, filename: &Path) -> io::Result<()> {
        let mut wtr = Writer::from_path(filename).expect("Bestand kon niet gemaakt worden!");

        for row in self.iter() {
            wtr.serialize(row)?
        }

        wtr.flush()
    }
}
