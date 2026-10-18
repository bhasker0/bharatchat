import 'dart:math';
import 'dart:typed_data';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:audioplayers/audioplayers.dart';

class SoundService {
  static final SoundService _instance = SoundService._internal();
  factory SoundService() => _instance;
  SoundService._internal();

  AudioPlayer? _player;
  AudioPlayer? _ringPlayer;

  AudioPlayer get player {
    _player ??= AudioPlayer();
    return _player!;
  }

  AudioPlayer get ringPlayer {
    _ringPlayer ??= AudioPlayer();
    return _ringPlayer!;
  }

  bool isSoundEnabled = true;
  bool _isRinging = false;

  /// Generate an in-memory 16-bit mono 44.1kHz WAV buffer for tone frequencies
  static Uint8List _generateWav(List<double> frequencies, List<double> durationsMs) {
    const int sampleRate = 44100;
    int totalSamples = 0;
    for (final d in durationsMs) {
      totalSamples += (sampleRate * (d / 1000.0)).round();
    }

    final int dataSize = totalSamples * 2;
    final int fileSize = 36 + dataSize;
    final bytes = ByteData(44 + dataSize);

    // RIFF chunk descriptor
    bytes.setUint8(0, 0x52); // R
    bytes.setUint8(1, 0x49); // I
    bytes.setUint8(2, 0x46); // F
    bytes.setUint8(3, 0x46); // F
    bytes.setUint32(4, fileSize, Endian.little);
    bytes.setUint8(8, 0x57);  // W
    bytes.setUint8(9, 0x41);  // A
    bytes.setUint8(10, 0x56); // V
    bytes.setUint8(11, 0x45); // E

    // "fmt " sub-chunk
    bytes.setUint8(12, 0x66); // f
    bytes.setUint8(13, 0x6D); // m
    bytes.setUint8(14, 0x74); // t
    bytes.setUint8(15, 0x20); // ' '
    bytes.setUint32(16, 16, Endian.little); // Subchunk1Size (16 for PCM)
    bytes.setUint16(20, 1, Endian.little);  // AudioFormat (1 for PCM)
    bytes.setUint16(22, 1, Endian.little);  // NumChannels (1 mono)
    bytes.setUint32(24, sampleRate, Endian.little);
    bytes.setUint32(28, sampleRate * 2, Endian.little); // ByteRate
    bytes.setUint16(32, 2, Endian.little); // BlockAlign
    bytes.setUint16(34, 16, Endian.little); // BitsPerSample

    // "data" sub-chunk
    bytes.setUint8(36, 0x64); // d
    bytes.setUint8(37, 0x61); // a
    bytes.setUint8(38, 0x74); // t
    bytes.setUint8(39, 0x61); // a
    bytes.setUint32(40, dataSize, Endian.little);

    int byteOffset = 44;
    for (int t = 0; t < frequencies.len(); t++) {
      final freq = frequencies[t];
      final durMs = durationsMs[t];
      final numSamples = (sampleRate * (durMs / 1000.0)).round();

      for (int i = 0; i < numSamples; i++) {
        // Apply envelope (fade in / fade out to avoid clicks)
        double envelope = 1.0;
        final fadeLen = min(200, numSamples ~/ 4);
        if (i < fadeLen) {
          envelope = i / fadeLen;
        } else if (i > numSamples - fadeLen) {
          envelope = (numSamples - i) / fadeLen;
        }

        final double sample = sin(2 * pi * freq * (i / sampleRate)) * envelope * 0.7;
        final int val = (sample * 32767.0).clamp(-32768, 32767).toInt();
        bytes.setInt16(byteOffset, val, Endian.little);
        byteOffset += 2;
      }
    }

    return bytes.buffer.asUint8List();
  }

  /// Play outgoing message sent tone (smooth dual-tone chime: 587Hz -> 880Hz)
  Future<void> playMessageSent() async {
    if (!isSoundEnabled) return;
    try {
      SystemSound.play(SystemSoundType.click);
      final wav = _generateWav([587.33, 880.0], [70.0, 90.0]);
      await player.play(BytesSource(wav));
    } catch (_) {
      try {
        SystemSound.play(SystemSoundType.click);
      } catch (_) {}
    }
  }

  /// Play incoming message pop tone (crisp notification: 784Hz -> 1046Hz)
  Future<void> playMessageReceived() async {
    if (!isSoundEnabled) return;
    try {
      SystemSound.play(SystemSoundType.click);
      final wav = _generateWav([783.99, 1046.50], [90.0, 140.0]);
      await player.play(BytesSource(wav));
    } catch (_) {
      try {
        SystemSound.play(SystemSoundType.click);
      } catch (_) {}
    }
  }

  /// Play active call dialer ringing sequence (repeats until answered or hung up)
  Future<void> startRinging() async {
    if (!isSoundEnabled || _isRinging) return;
    _isRinging = true;

    try {
      // Standard telephone dial ring tone (440Hz for 1s, silent 2s)
      final wav = _generateWav([440.0, 0.0], [1000.0, 2000.0]);
      await ringPlayer.setReleaseMode(ReleaseMode.loop);
      await ringPlayer.play(BytesSource(wav));
    } catch (_) {}
  }

  /// Stop ringing
  Future<void> stopRinging() async {
    _isRinging = false;
    try {
      await _ringPlayer?.stop();
    } catch (_) {}
  }

  /// Play call ended soft tone (descending hangup tone: 880Hz -> 659Hz -> 440Hz)
  Future<void> playCallEnded() async {
    await stopRinging();
    if (!isSoundEnabled) return;
    try {
      final wav = _generateWav([880.0, 659.25, 440.0], [120.0, 120.0, 200.0]);
      await player.play(BytesSource(wav));
    } catch (_) {
      try {
        SystemSound.play(SystemSoundType.click);
      } catch (_) {}
    }
  }

  void dispose() {
    _player?.dispose();
    _ringPlayer?.dispose();
  }
}

extension on List<double> {
  int len() => length;
}
