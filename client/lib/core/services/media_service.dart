import 'dart:async';
import 'dart:io' as io;
import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:image/image.dart' as img;
import 'package:path_provider/path_provider.dart';
import 'package:record/record.dart';

class MediaService {
  static final MediaService _instance = MediaService._internal();
  factory MediaService() => _instance;
  MediaService._internal();

  AudioRecorder? _audioRecorder;
  AudioRecorder get recorder {
    _audioRecorder ??= AudioRecorder();
    return _audioRecorder!;
  }

  bool _isRecording = false;
  DateTime? _recordStartTime;
  String? _currentRecordingPath;
  final List<int> _webAudioChunks = [];
  StreamSubscription<Uint8List>? _webStreamSub;

  bool get isRecording => _isRecording;
  int get recordingDurationSeconds =>
      _recordStartTime != null ? DateTime.now().difference(_recordStartTime!).inSeconds : 0;

  /// Pick an image from device and compress it with high visual fidelity:
  /// - Max dimension: 1600px (width or height)
  /// - Quality: 78% JPEG
  /// Returns a map with filename, bytes, mimeType, and compression metrics
  Future<Map<String, dynamic>?> pickAndCompressImage() async {
    try {
      final result = await FilePicker.pickFiles(
        type: FileType.image,
        allowMultiple: false,
        withData: true,
      );

      if (result == null || result.files.isEmpty) return null;

      final picked = result.files.first;
      final Uint8List rawBytes = picked.bytes ?? await picked.xFile.readAsBytes();
      if (rawBytes.isEmpty) return null;

      final originalSize = rawBytes.length;

      // Decode image using pure Dart image package
      final decoded = img.decodeImage(rawBytes);
      if (decoded == null) {
        // Fallback: return raw bytes if decode fails
        return {
          'filename': picked.name,
          'bytes': rawBytes.toList(),
          'mimeType': 'image/jpeg',
          'mediaType': 'IMAGE',
          'originalSize': originalSize,
          'compressedSize': originalSize,
        };
      }

      img.Image processed = decoded;

      // Resize if either dimension exceeds 1600px while maintaining aspect ratio
      const int maxDim = 1600;
      if (decoded.width > maxDim || decoded.height > maxDim) {
        if (decoded.width >= decoded.height) {
          processed = img.copyResize(
            decoded,
            width: maxDim,
            maintainAspect: true,
            interpolation: img.Interpolation.linear,
          );
        } else {
          processed = img.copyResize(
            decoded,
            height: maxDim,
            maintainAspect: true,
            interpolation: img.Interpolation.linear,
          );
        }
      }

      // Encode as high-quality JPEG (78%)
      final compressedBytes = img.encodeJpg(processed, quality: 78);
      final cleanBaseName = picked.name.replaceAll(RegExp(r'\.[^.]+$'), '');
      final cleanFilename = '${cleanBaseName}_compressed.jpg';

      return {
        'filename': cleanFilename,
        'bytes': compressedBytes.toList(),
        'mimeType': 'image/jpeg',
        'mediaType': 'IMAGE',
        'originalSize': originalSize,
        'compressedSize': compressedBytes.length,
      };
    } catch (e) {
      debugPrint('[MediaService] Error picking/compressing image: $e');
      return null;
    }
  }

  /// Pick any document from device and read at 100% FULL ORIGINAL SIZE (no compression)
  Future<Map<String, dynamic>?> pickDocument() async {
    try {
      final result = await FilePicker.pickFiles(
        type: FileType.any,
        allowMultiple: false,
        withData: true,
      );

      if (result == null || result.files.isEmpty) return null;

      final picked = result.files.first;
      final Uint8List rawBytes = picked.bytes ?? await picked.xFile.readAsBytes();
      if (rawBytes.isEmpty) return null;

      final mimeType = _resolveMimeType(picked.name);

      return {
        'filename': picked.name,
        'bytes': rawBytes.toList(),
        'mimeType': mimeType,
        'mediaType': 'DOCUMENT',
        'size': rawBytes.length,
      };
    } catch (e) {
      debugPrint('[MediaService] Error picking document: $e');
      return null;
    }
  }

  /// Start recording voice note from device microphone
  Future<bool> startRecording() async {
    try {
      final hasPermission = await recorder.hasPermission();
      if (!hasPermission) {
        debugPrint('[MediaService] Microphone permission not granted');
        return false;
      }

      _isRecording = true;
      _recordStartTime = DateTime.now();

      const config = RecordConfig(
        encoder: AudioEncoder.aacLc,
        bitRate: 128000,
        sampleRate: 44100,
      );

      if (kIsWeb) {
        _webAudioChunks.clear();
        final stream = await recorder.startStream(config);
        _webStreamSub = stream.listen((chunk) {
          _webAudioChunks.addAll(chunk);
        });
      } else {
        final tempDir = await getTemporaryDirectory();
        final path = '${tempDir.path}/voice_${DateTime.now().millisecondsSinceEpoch}.m4a';
        _currentRecordingPath = path;
        await recorder.start(config, path: path);
      }

      return true;
    } catch (e) {
      debugPrint('[MediaService] Error starting audio recording: $e');
      _isRecording = false;
      return false;
    }
  }

  /// Stop audio recording and return the recorded raw audio bytes
  Future<Map<String, dynamic>?> stopRecording() async {
    if (!_isRecording) return null;

    try {
      _isRecording = false;
      final durationSec = recordingDurationSeconds;
      _recordStartTime = null;

      List<int> audioBytes = [];

      if (kIsWeb) {
        await recorder.stop();
        await _webStreamSub?.cancel();
        _webStreamSub = null;
        audioBytes = List<int>.from(_webAudioChunks);
        _webAudioChunks.clear();
      } else {
        final stoppedPath = await recorder.stop();
        final targetPath = stoppedPath ?? _currentRecordingPath;
        if (targetPath != null) {
          final file = io.File(targetPath);
          if (await file.exists()) {
            audioBytes = await file.readAsBytes();
          }
        }
      }

      if (audioBytes.isEmpty) {
        debugPrint('[MediaService] Recorded audio bytes were empty');
        return null;
      }

      final filename = 'voice_${DateTime.now().millisecondsSinceEpoch}.m4a';

      return {
        'filename': filename,
        'bytes': audioBytes,
        'mimeType': 'audio/mp4',
        'mediaType': 'VOICE_NOTE',
        'durationSeconds': durationSec > 0 ? durationSec : 1,
        'size': audioBytes.length,
      };
    } catch (e) {
      debugPrint('[MediaService] Error stopping audio recording: $e');
      return null;
    }
  }

  /// Cancel and discard ongoing recording
  Future<void> cancelRecording() async {
    try {
      _isRecording = false;
      _recordStartTime = null;
      await _webStreamSub?.cancel();
      _webStreamSub = null;
      _webAudioChunks.clear();
      await recorder.cancel();
    } catch (_) {}
  }

  void dispose() {
    _audioRecorder?.dispose();
    _webStreamSub?.cancel();
  }

  static String _resolveMimeType(String filename) {
    final ext = filename.split('.').last.toLowerCase();
    switch (ext) {
      case 'jpg':
      case 'jpeg':
        return 'image/jpeg';
      case 'png':
        return 'image/png';
      case 'webp':
        return 'image/webp';
      case 'gif':
        return 'image/gif';
      case 'pdf':
        return 'application/pdf';
      case 'txt':
        return 'text/plain';
      case 'doc':
        return 'application/msword';
      case 'docx':
        return 'application/vnd.openxmlformats-officedocument.wordprocessingml.document';
      case 'xls':
        return 'application/vnd.ms-excel';
      case 'xlsx':
        return 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet';
      case 'zip':
        return 'application/zip';
      case 'tar':
      case 'gz':
        return 'application/gzip';
      case 'm4a':
      case 'aac':
        return 'audio/mp4';
      case 'mp3':
        return 'audio/mpeg';
      case 'wav':
        return 'audio/wav';
      default:
        return 'application/octet-stream';
    }
  }
}
