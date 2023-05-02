# Donation Calculator

A simple CLI tool to calculate the amount Typeractive.xyz should donate.

This is implemented in Rust and Zig to compare the two languages.

 - Retrieves in a list of orders from an inputted month from Shopify
 - Filters out refunded or replacement orders
 - Totals the per-order Typeractive.xyz donation (max[$1, 1%)])
 - Totals donations received (labeled as tips by Shopify), which are then matched by Typeractive.xyz
 - Outputs the donation for the given month
