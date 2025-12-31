import wave
import math
import struct
import random
import os

SAMPLE_RATE = 44100

def save_wav(filename, data):
    with wave.open(filename, 'w') as f:
        f.setnchannels(1)
        f.setsampwidth(2)
        f.setframerate(SAMPLE_RATE)
        for sample in data:
            # Clamp and convert to 16-bit PCM
            s = int(max(-1, min(1, sample)) * 32767)
            f.writeframes(struct.pack('<h', s))
    print(f"Generated {filename}")

def generate_square_wave(freq, duration, decay=False):
    samples = []
    num_samples = int(duration * SAMPLE_RATE)
    for i in range(num_samples):
        t = float(i) / SAMPLE_RATE
        val = 1.0 if math.sin(2 * math.pi * freq * t) > 0 else -1.0
        if decay:
            val *= (1.0 - t/duration)
        samples.append(val * 0.5)
    return samples

def generate_noise(duration, decay=True):
    samples = []
    num_samples = int(duration * SAMPLE_RATE)
    for i in range(num_samples):
        val = random.uniform(-1, 1)
        if decay:
            val *= (1.0 - float(i)/num_samples)
        samples.append(val * 0.5)
    return samples

def generate_laser(start_freq, end_freq, duration):
    samples = []
    num_samples = int(duration * SAMPLE_RATE)
    for i in range(num_samples):
        t = float(i) / SAMPLE_RATE
        progress = i / num_samples
        freq = start_freq + (end_freq - start_freq) * progress
        # Integrate frequency to get phase
        # Simplification: just use current freq for instantaneous phase step
        # Ideally: phase += freq / sample_rate
        # Let's do phase accumulation for smoother sweep
        pass
    
    # Redo with phase accumulator
    phase = 0
    for i in range(num_samples):
        progress = i / num_samples
        freq = start_freq + (end_freq - start_freq) * progress
        phase += freq / SAMPLE_RATE
        val = 1.0 if math.sin(2 * math.pi * phase) > 0 else -1.0 # Square wave laser
        samples.append(val * 0.3)
    return samples

def generate_sine(freq, duration, decay=False):
    samples = []
    num_samples = int(duration * SAMPLE_RATE)
    for i in range(num_samples):
        t = float(i) / SAMPLE_RATE
        val = math.sin(2 * math.pi * freq * t)
        if decay:
            val *= (1.0 - t/duration)
        samples.append(val * 0.5)
    return samples

def generate_engine(freq=100, duration=1.0):
    # Sawtooth-ish low rumble
    samples = []
    num_samples = int(duration * SAMPLE_RATE)
    phase = 0
    for i in range(num_samples):
        phase += freq / SAMPLE_RATE
        if phase > 1.0: phase -= 1.0
        val = (phase * 2.0 - 1.0) * 0.3
        samples.append(val)
    return samples

def mix_sounds(s1, s2):
    length = max(len(s1), len(s2))
    samples = []
    for i in range(length):
        v1 = s1[i] if i < len(s1) else 0
        v2 = s2[i] if i < len(s2) else 0
        samples.append(v1 + v2)
    return samples

# Ensure directory exists
os.makedirs("assets/sounds", exist_ok=True)

# 1. Repair: High pitched double ping
ping1 = generate_sine(880, 0.1, True) # A5
ping2 = generate_sine(1760, 0.2, True) # A6
save_wav("assets/sounds/repair.wav", ping1 + ping2)

# 2. Enemy Killed: Short noise burst
save_wav("assets/sounds/enemy_killed.wav", generate_noise(0.15, True))

# 3. Damage: Low crunch
save_wav("assets/sounds/damage.wav", generate_square_wave(100, 0.2, True))

# 4. Explosion: Longer noise decay
save_wav("assets/sounds/explosion.wav", generate_noise(0.5, True))

# 5. Laser: Fast sweep
save_wav("assets/sounds/laser.wav", generate_laser(800, 200, 0.15))

# 6. Pickup: Happy chime (major triad)
c5 = generate_sine(523.25, 0.1, True)
e5 = generate_sine(659.25, 0.1, True)
g5 = generate_sine(783.99, 0.15, True)
save_wav("assets/sounds/pickup.wav", c5 + e5 + g5)

# 7. Button Click: Very short high pip
save_wav("assets/sounds/click.wav", generate_sine(2000, 0.05, True))

# 8. Engine: Low rumble loop
save_wav("assets/sounds/engine.wav", generate_engine(60, 2.0))

# 9. Victory: Fanfare
c4 = generate_square_wave(261.63, 0.15, True)
e4 = generate_square_wave(329.63, 0.15, True)
g4 = generate_square_wave(392.00, 0.15, True)
c5_long = generate_square_wave(523.25, 0.6, True)
save_wav("assets/sounds/victory.wav", c4 + e4 + g4 + c5_long)

# 10. Game Over: Sad descending
g4_sad = generate_square_wave(392.00, 0.3, True)
e4_sad = generate_square_wave(311.13, 0.3, True) # Eb
c4_sad = generate_square_wave(261.63, 0.8, True)
save_wav("assets/sounds/gameover.wav", g4_sad + e4_sad + c4_sad)

print("All sounds generated in assets/sounds/")
