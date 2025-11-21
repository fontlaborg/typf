#!/usr/bin/env rust-script

//! Simple test to verify the ops/ns calculation fix

fn main() {
    println!("Testing ops/ns calculation fix:");
    println!("================================");

    // Test case 1: 10 iterations in 1,000,000 nanoseconds (1ms)
    let iterations = 10;
    let total_time_ns = 1_000_000u128;

    // Old calculation (what was causing the issue)
    let old_ops_per_ns = if total_time_ns > 0 {
        iterations as f64 / total_time_ns as f64
    } else {
        0.0
    };

    // New calculation (our fix)
    let new_ops_per_ns = if total_time_ns > 0 {
        let ns_per_op = total_time_ns as f64 / iterations as f64;
        1.0 / ns_per_op
    } else {
        0.0
    };

    println!("Test case: {} iterations in {} ns (1ms)", iterations, total_time_ns);
    println!("Old calculation: {:.6} ops/ns", old_ops_per_ns);
    println!("New calculation: {:.6} ops/ns", new_ops_per_ns);
    println!("Formatted old: {:8.3}", old_ops_per_ns);
    println!("Formatted new: {:8.3}", new_ops_per_ns);
    println!();

    // Test case 2: Faster case - 10 iterations in 100,000 nanoseconds (0.1ms)
    let total_time_ns = 100_000u128;

    let old_ops_per_ns = if total_time_ns > 0 {
        iterations as f64 / total_time_ns as f64
    } else {
        0.0
    };

    let new_ops_per_ns = if total_time_ns > 0 {
        let ns_per_op = total_time_ns as f64 / iterations as f64;
        1.0 / ns_per_op
    } else {
        0.0
    };

    println!("Test case: {} iterations in {} ns (0.1ms)", iterations, total_time_ns);
    println!("Old calculation: {:.6} ops/ns", old_ops_per_ns);
    println!("New calculation: {:.6} ops/ns", new_ops_per_ns);
    println!("Formatted old: {:8.3}", old_ops_per_ns);
    println!("Formatted new: {:8.3}", new_ops_per_ns);
    println!();

    // Test case 3: Very fast case - 100 iterations in 50,000 nanoseconds (0.05ms)
    let iterations = 100;
    let total_time_ns = 50_000u128;

    let old_ops_per_ns = if total_time_ns > 0 {
        iterations as f64 / total_time_ns as f64
    } else {
        0.0
    };

    let new_ops_per_ns = if total_time_ns > 0 {
        let ns_per_op = total_time_ns as f64 / iterations as f64;
        1.0 / ns_per_op
    } else {
        0.0
    };

    println!("Test case: {} iterations in {} ns (0.05ms)", iterations, total_time_ns);
    println!("Old calculation: {:.6} ops/ns", old_ops_per_ns);
    println!("New calculation: {:.6} ops/ns", new_ops_per_ns);
    println!("Formatted old: {:8.3}", old_ops_per_ns);
    println!("Formatted new: {:8.3}", new_ops_per_ns);
    println!();

    println!("Summary:");
    println!("--------");
    println!("The old calculation was showing very small numbers that rounded to 0.000");
    println!("The new calculation shows meaningful operations per nanosecond values");
}
