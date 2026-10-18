import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/network/api_client.dart';

class ProductItemModel {
  final String id;
  final String ownerDid;
  final String ownerName;
  final String title;
  final String description;
  final double priceInr;
  final String category;
  final String sku;
  final int createdAt;

  const ProductItemModel({
    required this.id,
    required this.ownerDid,
    required this.ownerName,
    required this.title,
    required this.description,
    required this.priceInr,
    required this.category,
    required this.sku,
    this.createdAt = 0,
  });

  factory ProductItemModel.fromJson(Map<String, dynamic> json) {
    return ProductItemModel(
      id: json['id'] as String? ?? 'prod_${DateTime.now().millisecondsSinceEpoch}',
      ownerDid: json['owner_did'] as String? ?? '',
      ownerName: json['owner_name'] as String? ?? 'Sovereign Merchant',
      title: json['title'] as String? ?? 'Untitled Product',
      description: json['description'] as String? ?? '',
      priceInr: (json['price_inr'] as num?)?.toDouble() ?? 0.0,
      category: json['category'] as String? ?? 'General',
      sku: json['sku'] as String? ?? '',
      createdAt: json['created_at'] as int? ?? 0,
    );
  }

  Map<String, dynamic> toJsonLd() {
    return {
      "@context": "https://schema.org/",
      "@type": "Product",
      "identifier": id,
      "sku": sku,
      "name": title,
      "description": description,
      "offers": {
        "@type": "Offer",
        "priceCurrency": "INR",
        "price": priceInr.toStringAsFixed(2),
        "availability": "https://schema.org/InStock",
      }
    };
  }
}

class CatalogState {
  final List<ProductItemModel> products;
  final Map<String, int> cart;
  final bool isLoading;
  final String? error;

  const CatalogState({
    required this.products,
    required this.cart,
    this.isLoading = false,
    this.error,
  });

  CatalogState copyWith({
    List<ProductItemModel>? products,
    Map<String, int>? cart,
    bool? isLoading,
    String? error,
  }) {
    return CatalogState(
      products: products ?? this.products,
      cart: cart ?? this.cart,
      isLoading: isLoading ?? this.isLoading,
      error: error,
    );
  }
}

class CatalogNotifier extends StateNotifier<CatalogState> {
  final ApiClient _apiClient;

  CatalogNotifier([ApiClient? apiClient])
      : _apiClient = apiClient ?? ApiClient(),
        super(const CatalogState(products: [], cart: {})) {
    loadProducts();
  }

  /// Load products from backend gateway
  Future<void> loadProducts() async {
    state = state.copyWith(isLoading: true, error: null);
    try {
      final rawList = await _apiClient.getProducts();
      if (rawList.isNotEmpty) {
        final prods = rawList.map((m) => ProductItemModel.fromJson(m)).toList();
        state = state.copyWith(products: prods, isLoading: false);
      } else {
        // Fallback default sample seed
        state = state.copyWith(
          products: [
            const ProductItemModel(
              id: "prod_001",
              ownerDid: "did:bharat:9876543210",
              ownerName: "Ramesh Patel",
              title: "Pure Kanjivaram Silk Saree",
              description: "Authentic handloom woven silk with 2.5g gold zari borders",
              priceInr: 4500.0,
              category: "Textiles",
              sku: "KANJI-001",
            ),
            const ProductItemModel(
              id: "prod_002",
              ownerDid: "did:bharat:9876543210",
              ownerName: "Ramesh Patel",
              title: "Surat Banarasi Georgette Saree",
              description: "Lightweight festive handwoven floral motifs",
              priceInr: 3200.0,
              category: "Textiles",
              sku: "BANARAS-002",
            ),
            const ProductItemModel(
              id: "prod_003",
              ownerDid: "did:bharat:9811122233",
              ownerName: "Priya Sharma",
              title: "Chanderi Cotton Embroidered Kurti",
              description: "Breathable pure organic cotton summer collection",
              priceInr: 1200.0,
              category: "Ready-to-wear",
              sku: "CHAND-003",
            ),
          ],
          isLoading: false,
        );
      }
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
    }
  }

  /// Add new product for seller
  Future<bool> addProduct({
    required String ownerDid,
    required String ownerName,
    required String title,
    required String description,
    required double priceInr,
    required String category,
    required String sku,
  }) async {
    final res = await _apiClient.createProduct(
      ownerDid: ownerDid,
      ownerName: ownerName,
      title: title,
      description: description,
      priceInr: priceInr,
      category: category,
      sku: sku,
    );

    if (res['success'] == true) {
      await loadProducts();
      return true;
    }
    return false;
  }

  /// Update an existing product (only owner)
  Future<bool> updateProduct({
    required String id,
    String? title,
    String? description,
    double? priceInr,
    String? category,
    String? sku,
  }) async {
    final res = await _apiClient.updateProduct(
      id: id,
      title: title,
      description: description,
      priceInr: priceInr,
      category: category,
      sku: sku,
    );

    if (res['success'] == true) {
      await loadProducts();
      return true;
    }
    return false;
  }

  /// Delete a product (only owner)
  Future<bool> deleteProduct(String id) async {
    final ok = await _apiClient.deleteProduct(id);
    if (ok) {
      state = state.copyWith(
        products: state.products.where((p) => p.id != id).toList(),
      );
      return true;
    }
    return false;
  }

  /// Cart controls
  void addToCart(String id) {
    final updatedCart = Map<String, int>.from(state.cart);
    updatedCart[id] = (updatedCart[id] ?? 0) + 1;
    state = state.copyWith(cart: updatedCart);
  }

  void removeFromCart(String id) {
    final updatedCart = Map<String, int>.from(state.cart);
    if (updatedCart.containsKey(id)) {
      if (updatedCart[id]! > 1) {
        updatedCart[id] = updatedCart[id]! - 1;
      } else {
        updatedCart.remove(id);
      }
      state = state.copyWith(cart: updatedCart);
    }
  }

  void clearCart() {
    state = state.copyWith(cart: {});
  }
}

final catalogProvider = StateNotifierProvider<CatalogNotifier, CatalogState>((ref) {
  return CatalogNotifier(ref.watch(apiClientProvider));
});
