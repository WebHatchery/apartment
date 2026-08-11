use super::*;

#[test]
fn test_listing_generation() {
    let neighborhood = Neighborhood::new(0, NeighborhoodType::Downtown, "Test");
    let listing = PropertyListing::generate(0, &neighborhood);

    assert!(listing.asking_price > 0);
    assert!(listing.num_floors >= 2);
}

#[test]
fn test_financing_calculations() {
    let mortgage = FinancingOption::Mortgage {
        down_payment_percent: 0.2,
        interest_rate: 0.06,
        term_months: 120,
    };

    let upfront = mortgage.upfront_cost(100000);
    assert_eq!(upfront, 20000); // 20% down

    let monthly = mortgage.monthly_payment(100000);
    assert!(monthly > 0 && monthly < 2000); // Reasonable range
}
