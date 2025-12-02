#!/usr/bin/env python3
"""
Analyze integrated energy harvester simulation results.

This script visualizes:
1. Energy evolution at each component over time
2. Amplification factor vs time
3. Component dimensional field evolution
4. Coupling strength between components
5. Resonance analysis (FFT of energy signals)
"""

import numpy as np
import matplotlib.pyplot as plt
import pandas as pd
from pathlib import Path
from scipy import signal
from scipy.fft import fft, fftfreq

# Configure matplotlib
plt.style.use('seaborn-v0_8-darkgrid')
plt.rcParams['figure.figsize'] = (14, 10)
plt.rcParams['font.size'] = 10

def load_data():
    """Load simulation results from CSV files."""
    print("Loading simulation data...")

    energy_file = Path("integrated_energy_evolution.csv")
    component_file = Path("component_interactions.csv")

    if not energy_file.exists():
        print(f"ERROR: {energy_file} not found!")
        return None, None

    energy_data = pd.read_csv(energy_file)
    component_data = pd.read_csv(component_file) if component_file.exists() else None

    print(f"  Loaded {len(energy_data)} timesteps")
    return energy_data, component_data

def plot_energy_evolution(energy_data):
    """Plot energy at each component over time."""
    print("\nCreating energy evolution plot...")

    fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(16, 12))

    time = energy_data['time'].values

    # Plot 1: All component energies
    ax1.plot(time, energy_data['fractal_energy'], 'b-', linewidth=2,
             label='Fractal Antenna', alpha=0.8)
    ax1.plot(time, energy_data['spiral_energy'], 'g-', linewidth=2,
             label='Spiral Concentrator', alpha=0.8)
    ax1.plot(time, energy_data['toroid_energy'], 'r-', linewidth=2,
             label='Toroidal Vortex', alpha=0.8)
    ax1.plot(time, energy_data['total_energy'], 'k--', linewidth=2,
             label='Total System', alpha=0.6)

    ax1.set_xlabel('Time (simulation units)', fontsize=12)
    ax1.set_ylabel('Energy Density', fontsize=12)
    ax1.set_title('Component Energy Evolution', fontsize=14, fontweight='bold')
    ax1.legend(fontsize=10)
    ax1.grid(True, alpha=0.3)

    # Plot 2: Amplification factor
    amplification = energy_data['amplification'].values
    ax2.plot(time, amplification, 'purple', linewidth=2)
    ax2.axhline(y=1.0, color='gray', linestyle='--', alpha=0.5, label='Baseline')

    max_amp = amplification.max()
    max_time = time[amplification.argmax()]
    ax2.plot(max_time, max_amp, 'ro', markersize=10,
             label=f'Peak: {max_amp:.2f}× at t={max_time:.1f}')

    ax2.set_xlabel('Time (simulation units)', fontsize=12)
    ax2.set_ylabel('Amplification Factor', fontsize=12)
    ax2.set_title('Energy Amplification Over Time', fontsize=14, fontweight='bold')
    ax2.legend(fontsize=10)
    ax2.grid(True, alpha=0.3)

    # Plot 3: Energy ratios (normalized to total)
    total = energy_data['total_energy'].values
    total_nonzero = np.where(total > 0, total, 1.0)  # Avoid div by zero

    fractal_frac = energy_data['fractal_energy'].values / total_nonzero * 100
    spiral_frac = energy_data['spiral_energy'].values / total_nonzero * 100
    toroid_frac = energy_data['toroid_energy'].values / total_nonzero * 100

    ax3.stackplot(time, fractal_frac, spiral_frac, toroid_frac,
                  labels=['Fractal', 'Spiral', 'Toroid'],
                  colors=['blue', 'green', 'red'], alpha=0.6)

    ax3.set_xlabel('Time (simulation units)', fontsize=12)
    ax3.set_ylabel('Energy Fraction (%)', fontsize=12)
    ax3.set_title('Relative Component Contributions', fontsize=14, fontweight='bold')
    ax3.legend(fontsize=10, loc='best')
    ax3.grid(True, alpha=0.3)
    ax3.set_ylim([0, 100])

    # Plot 4: Log scale energy
    ax4.semilogy(time, energy_data['fractal_energy'], 'b-', linewidth=2,
                 label='Fractal', alpha=0.8)
    ax4.semilogy(time, energy_data['spiral_energy'], 'g-', linewidth=2,
                 label='Spiral', alpha=0.8)
    ax4.semilogy(time, energy_data['toroid_energy'], 'r-', linewidth=2,
                 label='Toroid', alpha=0.8)

    ax4.set_xlabel('Time (simulation units)', fontsize=12)
    ax4.set_ylabel('Energy Density (log scale)', fontsize=12)
    ax4.set_title('Energy Evolution (Logarithmic Scale)', fontsize=14, fontweight='bold')
    ax4.legend(fontsize=10)
    ax4.grid(True, alpha=0.3, which='both')

    plt.tight_layout()
    plt.savefig('integrated_energy_evolution.png', dpi=150, bbox_inches='tight')
    print("  Saved: integrated_energy_evolution.png")

    return fig

def plot_component_fields(component_data):
    """Plot dimensional field values at each component."""
    if component_data is None:
        print("\nSkipping component fields plot (no data)")
        return None

    print("\nCreating component field evolution plot...")

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(14, 10))

    time = component_data['time'].values

    # Plot 1: Dimensional field values
    ax1.plot(time, component_data['fractal_dim'], 'b-', linewidth=2,
             label='Fractal d-field', alpha=0.8)
    ax1.plot(time, component_data['spiral_dim'], 'g-', linewidth=2,
             label='Spiral d-field', alpha=0.8)
    ax1.plot(time, component_data['toroid_dim'], 'r-', linewidth=2,
             label='Toroid d-field', alpha=0.8)

    # Background dimension line
    ax1.axhline(y=3.0, color='gray', linestyle='--', alpha=0.5,
                label='Background (d=3.0)')

    ax1.set_xlabel('Time (simulation units)', fontsize=12)
    ax1.set_ylabel('Dimensional Field d', fontsize=12)
    ax1.set_title('Dimensional Field Evolution at Components', fontsize=14, fontweight='bold')
    ax1.legend(fontsize=10)
    ax1.grid(True, alpha=0.3)

    # Plot 2: Coupling strength
    coupling = component_data['coupling_strength'].values
    ax2.plot(time, coupling, 'purple', linewidth=2)

    # Mark peaks in coupling
    from scipy.signal import find_peaks
    peaks, _ = find_peaks(coupling, height=coupling.mean())
    if len(peaks) > 0:
        ax2.plot(time[peaks], coupling[peaks], 'ro', markersize=8,
                 label=f'{len(peaks)} resonance peaks')

    ax2.set_xlabel('Time (simulation units)', fontsize=12)
    ax2.set_ylabel('Coupling Strength', fontsize=12)
    ax2.set_title('Inter-Component Coupling', fontsize=14, fontweight='bold')
    ax2.legend(fontsize=10)
    ax2.grid(True, alpha=0.3)

    plt.tight_layout()
    plt.savefig('component_field_evolution.png', dpi=150, bbox_inches='tight')
    print("  Saved: component_field_evolution.png")

    return fig

def analyze_resonances(energy_data):
    """Perform FFT to find resonant frequencies."""
    print("\nAnalyzing resonances (FFT)...")

    fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(16, 12))

    time = energy_data['time'].values
    dt = time[1] - time[0]  # Time step
    N = len(time)

    # FFT of each component
    components = {
        'Fractal': energy_data['fractal_energy'].values,
        'Spiral': energy_data['spiral_energy'].values,
        'Toroid': energy_data['toroid_energy'].values,
        'Total': energy_data['total_energy'].values,
    }

    freqs = fftfreq(N, dt)[:N//2]  # Positive frequencies only

    colors = {'Fractal': 'blue', 'Spiral': 'green', 'Toroid': 'red', 'Total': 'black'}
    axes = [ax1, ax2, ax3, ax4]

    for idx, (name, signal_data) in enumerate(components.items()):
        ax = axes[idx]
        color = colors[name]

        # Remove DC component
        signal_data = signal_data - signal_data.mean()

        # Compute FFT
        fft_vals = fft(signal_data)
        power_spectrum = np.abs(fft_vals[:N//2])**2

        # Normalize
        power_spectrum /= power_spectrum.max()

        # Plot
        ax.plot(freqs, power_spectrum, color=color, linewidth=2, alpha=0.8)

        # Find peaks
        from scipy.signal import find_peaks
        peaks, properties = find_peaks(power_spectrum, height=0.1, distance=10)

        if len(peaks) > 0:
            ax.plot(freqs[peaks], power_spectrum[peaks], 'ro', markersize=8)

            # Annotate top 3 peaks
            top_peaks = peaks[np.argsort(power_spectrum[peaks])[-3:]]
            for peak in top_peaks:
                freq_val = freqs[peak]
                power_val = power_spectrum[peak]
                if power_val > 0.2:  # Only label significant peaks
                    ax.annotate(f'f={freq_val:.3f}',
                               xy=(freq_val, power_val),
                               xytext=(10, 10), textcoords='offset points',
                               fontsize=8, ha='left',
                               bbox=dict(boxstyle='round,pad=0.3', fc='yellow', alpha=0.5),
                               arrowprops=dict(arrowstyle='->', connectionstyle='arc3,rad=0'))

        ax.set_xlabel('Frequency (sim units)', fontsize=10)
        ax.set_ylabel('Normalized Power', fontsize=10)
        ax.set_title(f'{name} - Frequency Spectrum', fontsize=12, fontweight='bold')
        ax.grid(True, alpha=0.3)
        ax.set_xlim([0, 10])  # Focus on low frequencies

    plt.tight_layout()
    plt.savefig('resonance_analysis.png', dpi=150, bbox_inches='tight')
    print("  Saved: resonance_analysis.png")

    # Print resonance frequencies
    print("\n  Detected resonance frequencies:")
    for name, signal_data in components.items():
        signal_data = signal_data - signal_data.mean()
        fft_vals = fft(signal_data)
        power_spectrum = np.abs(fft_vals[:N//2])**2

        from scipy.signal import find_peaks
        peaks, _ = find_peaks(power_spectrum, height=0.1*power_spectrum.max())

        if len(peaks) > 0:
            top_peaks = peaks[np.argsort(power_spectrum[peaks])[-3:]][::-1]
            print(f"    {name}:", end=' ')
            for peak in top_peaks:
                print(f"f={freqs[peak]:.3f}", end=' ')
            print()

    return fig

def compute_statistics(energy_data, component_data):
    """Compute and display statistical summary."""
    print("\n" + "="*60)
    print("INTEGRATED HARVESTER STATISTICS")
    print("="*60)

    # Energy statistics
    print("\nEnergy Statistics:")

    components = ['fractal_energy', 'spiral_energy', 'toroid_energy', 'total_energy']
    names = ['Fractal', 'Spiral', 'Toroid', 'Total']

    for comp, name in zip(components, names):
        data = energy_data[comp].values
        print(f"\n  {name}:")
        print(f"    Mean:   {data.mean():.6f}")
        print(f"    Std:    {data.std():.6f}")
        print(f"    Min:    {data.min():.6f}")
        print(f"    Max:    {data.max():.6f}")
        print(f"    Final:  {data[-1]:.6f}")

    # Amplification statistics
    print("\nAmplification Factor:")
    amp = energy_data['amplification'].values
    print(f"  Mean:     {amp.mean():.3f}×")
    print(f"  Std:      {amp.std():.3f}×")
    print(f"  Maximum:  {amp.max():.3f}× (at t={energy_data['time'].values[amp.argmax()]:.1f})")
    print(f"  Final:    {amp[-1]:.3f}×")

    # Component contributions (final)
    final_total = energy_data['total_energy'].values[-1]
    if final_total > 0:
        print("\nFinal Component Contributions:")
        final_fractal = energy_data['fractal_energy'].values[-1] / final_total * 100
        final_spiral = energy_data['spiral_energy'].values[-1] / final_total * 100
        final_toroid = energy_data['toroid_energy'].values[-1] / final_total * 100

        print(f"  Fractal:  {final_fractal:.1f}%")
        print(f"  Spiral:   {final_spiral:.1f}%")
        print(f"  Toroid:   {final_toroid:.1f}%")

    # Component field statistics
    if component_data is not None:
        print("\nDimensional Field Statistics:")

        for comp in ['fractal_dim', 'spiral_dim', 'toroid_dim']:
            data = component_data[comp].values
            name = comp.replace('_dim', '').capitalize()
            deviation = data - 3.0  # Deviation from background

            print(f"\n  {name}:")
            print(f"    Mean:       {data.mean():.4f}")
            print(f"    Deviation:  {deviation.mean():.4f} ± {deviation.std():.4f}")
            print(f"    Range:      [{data.min():.4f}, {data.max():.4f}]")

        # Coupling statistics
        coupling = component_data['coupling_strength'].values
        print("\nCoupling Strength:")
        print(f"  Mean:   {coupling.mean():.6f}")
        print(f"  Max:    {coupling.max():.6f}")
        print(f"  Std:    {coupling.std():.6f}")

def main():
    """Main analysis routine."""
    print("="*60)
    print("INTEGRATED ENERGY HARVESTER ANALYSIS")
    print("="*60)

    # Load data
    energy_data, component_data = load_data()
    if energy_data is None:
        print("\nERROR: Could not load simulation data. Exiting.")
        return

    # Create plots
    plot_energy_evolution(energy_data)
    plot_component_fields(component_data)
    analyze_resonances(energy_data)

    # Compute statistics
    compute_statistics(energy_data, component_data)

    print("\n" + "="*60)
    print("Analysis complete! Generated plots:")
    print("  - integrated_energy_evolution.png")
    print("  - component_field_evolution.png")
    print("  - resonance_analysis.png")
    print("="*60)

if __name__ == "__main__":
    main()
