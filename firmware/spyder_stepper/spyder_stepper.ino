/*
 * Spyder multi-axis stepper firmware (Arduino / ESP32).
 *
 * Protocol (newline-terminated):
 *   M <n> <steps0> <delay0_us> ... <stepsN-1> <delayN-1_us>
 *   H                          — hardware home / zero positions
 *   P                          — report positions: P s0 s1 ...
 *   E                          — e-stop / halt
 *
 * Reply: OK\n or ERR <msg>\n
 *
 * Wire STEP/DIR pins in the arrays below. Default: 4 axes.
 */

#ifndef SPYDER_MAX_AXES
#define SPYDER_MAX_AXES 8
#endif

const int NUM_AXES = 4;
const int STEP_PINS[SPYDER_MAX_AXES] = {2, 3, 4, 5};
const int DIR_PINS[SPYDER_MAX_AXES]  = {6, 7, 8, 9};
const int EN_PIN = -1; // set to a pin to enable drivers, or -1

long positions[SPYDER_MAX_AXES];

void setup() {
  Serial.begin(115200);
  for (int i = 0; i < NUM_AXES; i++) {
    pinMode(STEP_PINS[i], OUTPUT);
    pinMode(DIR_PINS[i], OUTPUT);
    positions[i] = 0;
  }
  if (EN_PIN >= 0) {
    pinMode(EN_PIN, OUTPUT);
    digitalWrite(EN_PIN, LOW); // active-low enable common
  }
  Serial.println("OK spyder-stepper");
}

void pulseAxis(int axis, long steps, unsigned long delay_us) {
  if (steps == 0) return;
  digitalWrite(DIR_PINS[axis], steps > 0 ? HIGH : LOW);
  long n = steps > 0 ? steps : -steps;
  if (delay_us < 2) delay_us = 2;
  for (long i = 0; i < n; i++) {
    digitalWrite(STEP_PINS[axis], HIGH);
    delayMicroseconds(2);
    digitalWrite(STEP_PINS[axis], LOW);
    if (delay_us > 2) delayMicroseconds(delay_us - 2);
  }
  positions[axis] += steps;
}

// Interleaved multi-axis move so cables stay roughly synchronized.
void moveSynced(int n, long *steps, unsigned long *delays_us) {
  long remaining[SPYDER_MAX_AXES];
  unsigned long last[SPYDER_MAX_AXES];
  unsigned long now = micros();
  long total = 0;
  for (int i = 0; i < n; i++) {
    remaining[i] = steps[i] > 0 ? steps[i] : -steps[i];
    digitalWrite(DIR_PINS[i], steps[i] >= 0 ? HIGH : LOW);
    last[i] = now;
    total += remaining[i];
  }
  while (total > 0) {
    now = micros();
    for (int i = 0; i < n; i++) {
      if (remaining[i] <= 0) continue;
      unsigned long d = delays_us[i] < 2 ? 2 : delays_us[i];
      if (now - last[i] >= d) {
        digitalWrite(STEP_PINS[i], HIGH);
        delayMicroseconds(2);
        digitalWrite(STEP_PINS[i], LOW);
        last[i] = now;
        remaining[i]--;
        total--;
        positions[i] += (steps[i] >= 0 ? 1 : -1);
      }
    }
  }
}

void loop() {
  if (!Serial.available()) return;
  String line = Serial.readStringUntil('\n');
  line.trim();
  if (line.length() == 0) return;

  if (line == "H") {
    for (int i = 0; i < NUM_AXES; i++) positions[i] = 0;
    Serial.println("OK");
    return;
  }
  if (line == "E") {
    // E-stop: stop issuing pulses (cooperative); host should cease commands.
    Serial.println("OK estop");
    return;
  }
  if (line == "P") {
    Serial.print("P");
    for (int i = 0; i < NUM_AXES; i++) {
      Serial.print(' ');
      Serial.print(positions[i]);
    }
    Serial.println();
    return;
  }
  if (line.charAt(0) == 'M') {
    // parse M n s0 d0 ...
    int start = 1;
    while (start < (int)line.length() && line.charAt(start) == ' ') start++;
    int n = 0;
    int idx = start;
    // read n
    while (idx < (int)line.length() && isDigit(line.charAt(idx))) {
      n = n * 10 + (line.charAt(idx) - '0');
      idx++;
    }
    if (n <= 0 || n > NUM_AXES) {
      Serial.println("ERR bad n");
      return;
    }
    long steps[SPYDER_MAX_AXES];
    unsigned long delays[SPYDER_MAX_AXES];
    for (int i = 0; i < n; i++) {
      while (idx < (int)line.length() && line.charAt(idx) == ' ') idx++;
      bool neg = false;
      if (idx < (int)line.length() && line.charAt(idx) == '-') { neg = true; idx++; }
      long v = 0;
      while (idx < (int)line.length() && isDigit(line.charAt(idx))) {
        v = v * 10 + (line.charAt(idx) - '0');
        idx++;
      }
      steps[i] = neg ? -v : v;
      while (idx < (int)line.length() && line.charAt(idx) == ' ') idx++;
      unsigned long d = 0;
      while (idx < (int)line.length() && isDigit(line.charAt(idx))) {
        d = d * 10 + (line.charAt(idx) - '0');
        idx++;
      }
      delays[i] = d;
    }
    moveSynced(n, steps, delays);
    Serial.println("OK");
    return;
  }
  Serial.println("ERR unknown");
}
