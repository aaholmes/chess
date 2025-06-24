#!/bin/bash

# Comprehensive Validation Experiments for Tactical-Enhanced MCTS
# This script runs all the experiments needed to validate the efficiency claims

echo "🧪 Starting Comprehensive Tactical-Enhanced MCTS Validation"
echo "============================================================"

# Build in release mode for accurate performance measurements
echo "🔨 Building in release mode..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed. Exiting."
    exit 1
fi

echo "✅ Build successful"

# Create results directory
mkdir -p validation_results
cd validation_results

echo ""
echo "🎯 Experiment 1: Tactical Efficiency Benchmark"
echo "----------------------------------------------"
echo "Comparing Tactical-Enhanced MCTS vs Classical MCTS vs Alpha-Beta"

# Run the comprehensive benchmark
../target/release/tactical_benchmark > tactical_efficiency_results.txt 2>&1

if [ $? -eq 0 ]; then
    echo "✅ Tactical efficiency benchmark completed"
    echo "📊 Results saved to validation_results/tactical_efficiency_results.txt"
else
    echo "⚠️  Tactical efficiency benchmark encountered issues"
fi

echo ""
echo "🔍 Experiment 2: Performance Profiling"
echo "--------------------------------------"
echo "Profiling computational bottlenecks and optimization opportunities"

# Run the performance profiler
../target/release/tactical_profiler > performance_profile_results.txt 2>&1

if [ $? -eq 0 ]; then
    echo "✅ Performance profiling completed"
    echo "📊 Results saved to validation_results/performance_profile_results.txt"
else
    echo "⚠️  Performance profiling encountered issues"
fi

echo ""
echo "🎮 Experiment 3: Quick Tactical Test"
echo "------------------------------------"
echo "Running basic tactical MCTS functionality test"

# Run the tactical test (our standalone test)
../test_tactical_mcts > basic_tactical_test.txt 2>&1

if [ $? -eq 0 ]; then
    echo "✅ Basic tactical test completed"
    echo "📊 Results saved to validation_results/basic_tactical_test.txt"
else
    echo "⚠️  Basic tactical test encountered issues"
fi

echo ""
echo "📈 Experiment 4: Strength Comparison"
echo "------------------------------------"
echo "Running existing strength tests for baseline comparison"

# Run existing strength tests if available
if [ -f "../target/release/strength_test" ]; then
    ../target/release/strength_test --time 1000 > strength_comparison.txt 2>&1
    
    if [ $? -eq 0 ]; then
        echo "✅ Strength comparison completed"
        echo "📊 Results saved to validation_results/strength_comparison.txt"
    else
        echo "⚠️  Strength comparison encountered issues"
    fi
else
    echo "⚠️  Strength test binary not found, skipping"
fi

echo ""
echo "📊 EXPERIMENT SUMMARY"
echo "===================="

echo "Results saved in validation_results/ directory:"
echo "  - tactical_efficiency_results.txt: Main efficiency comparison"
echo "  - performance_profile_results.txt: Performance bottleneck analysis"
echo "  - basic_tactical_test.txt: Basic functionality validation"
if [ -f "strength_comparison.txt" ]; then
    echo "  - strength_comparison.txt: Engine strength comparison"
fi

echo ""
echo "🔬 ANALYSIS INSTRUCTIONS"
echo "========================"
echo "1. Review tactical_efficiency_results.txt for NN call reduction metrics"
echo "2. Check performance_profile_results.txt for bottlenecks and optimization opportunities"
echo "3. Verify basic_tactical_test.txt shows successful move selection and statistics"

echo ""
echo "📝 PUBLICATION DATA EXTRACTION"
echo "=============================="
echo "Extract these key metrics from the results:"
echo "  - NN call reduction percentage (from tactical_efficiency_results.txt)"
echo "  - Nodes per second comparison (from performance_profile_results.txt)"
echo "  - Mate detection accuracy (from tactical_efficiency_results.txt)"
echo "  - Search time efficiency (from all result files)"

echo ""
echo "✅ All validation experiments completed!"
echo "Review the results files to validate the Tactics-Enhanced MCTS claims."

cd ..