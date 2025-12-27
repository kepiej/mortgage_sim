use std::f64;
mod mortgage;
mod paymentschemes;
use inquire::{Confirm, CustomType, Select, Text};
use mortgage::Mortgage;
use paymentschemes::{MortgagePayments, PaymentScheme};
use polars::io::csv::write::CsvWriter;
use polars::prelude::SerWriter;
use std::fs::File;
use std::io;

fn main() {
    let input: io::Stdin = io::stdin();
    let mut inbuffer = String::new();

    let capital: f64 = CustomType::<f64>::new("Kapitaal?")
        .with_help_message("Geef het te ontlenen bedrag in.")
        .prompt()
        .unwrap();
    let nyears: usize = CustomType::<usize>::new("Looptijd in jaar?")
        .with_help_message("Dit is de totale looptijd van de lening in jaren.")
        .prompt()
        .unwrap();

    let isfixed: &str = Select::new("Vast of variabel?", ["Vast", "Variabel"].to_vec())
        .with_help_message("Een vaste rentevoet blijft ongewijzigd gedurende de gehele looptijd. Een variabele rentevoet kan op vaste tijdstippen aangepast worden. In geval van een variabele rentevoet wordt het worst-case scenario gesimuleerd.")
        .prompt()
        .unwrap();
    let year_r: f64;
    let mut year_interest_rate: Vec<f64>;
    if isfixed == "Vast" {
        year_r = CustomType::<f64>::new("Jaarlijkse interestvoet?")
            .prompt()
            .unwrap();
        year_interest_rate = vec![year_r / 100.0; nyears * 12];
    } else {
        year_r = CustomType::<f64>::new("Initieele interestvoet?")
            .prompt()
            .unwrap();
        let first_month_revision: usize =
            CustomType::<usize>::new("Maand waarin rentevoet voor het eerst herzien wordt?")
                .prompt()
                .unwrap();
        let max_year_interest_rate: f64 =
            CustomType::<f64>::new("Maximale jaarlijkse interestvoet?")
                .prompt()
                .unwrap();
        year_interest_rate = vec![year_r / 100.0; first_month_revision - 1];
        year_interest_rate.append(&mut vec![
            max_year_interest_rate / 100.0;
            (nyears * 12) - first_month_revision + 1
        ])
    }

    let mort: Mortgage = Mortgage::new(capital, (nyears * 12) as i64, year_interest_rate);

    let payscheme_str: &str = Select::new(
        "Type lening?",
        [
            "VasteKapitaalaflossing",
            "VasteMensualiteiten",
            "VariabeleLineaireKapitaalaflossing",
        ]
        .to_vec(),
    )
    .prompt()
    .unwrap();

    let payscheme: PaymentScheme = match payscheme_str {
        "VariableLinearCapital" | "VariabeleLineaireKapitaalaflossing" => {
            println!("Initieele afbetaling?");
            let init_pay: f64 = match input.read_line(&mut inbuffer) {
                Ok(_) => inbuffer.trim().parse().unwrap(),
                Err(_) => panic!(),
            };
            PaymentScheme::VariableLinearCapital(init_pay)
        }
        _ => payscheme_str.parse().unwrap(),
    };

    let mortpay: MortgagePayments = MortgagePayments::new(mort, payscheme);

    println!("Total repayment: {}", mortpay.total_repaid());
    println!("{}", mortpay.to_pl());

    let save2file: bool = Confirm::new("Wil je de aflossingstabel bewaren in een bestand?")
        .with_default(true)
        .prompt()
        .unwrap();

    if save2file {
        let filename: String = Text::new("Bestandsnaam?")
            .with_default("hypotheeksimulatie.csv")
            .prompt()
            .unwrap();

        let mut file = File::create(filename).expect("Bestand kon niet gemaakt worden!");
        CsvWriter::new(&mut file)
            .include_header(true)
            .with_separator(b',')
            .finish(&mut mortpay.to_pl())
            .expect("Resultaten konden niet weggeschreven worden!");
    }
}
