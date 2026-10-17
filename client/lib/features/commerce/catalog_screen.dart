import 'package:flutter/material.dart';
import '../../core/theme/app_theme.dart';

class ProductItemModel {
  final String id;
  final String title;
  final String description;
  final double priceInr;
  final String category;
  final String sku;

  const ProductItemModel({
    required this.id,
    required this.title,
    required this.description,
    required this.priceInr,
    required this.category,
    required this.sku,
  });

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

class CatalogScreen extends StatefulWidget {
  const CatalogScreen({super.key});

  @override
  State<CatalogScreen> createState() => _CatalogScreenState();
}

class _CatalogScreenState extends State<CatalogScreen> {
  final List<ProductItemModel> products = const [
    ProductItemModel(
      id: "prod_001",
      title: "Pure Kanjivaram Silk Saree",
      description: "Authentic handloom woven silk with 2.5g gold zari borders",
      priceInr: 4500.0,
      category: "Textiles",
      sku: "KANJI-001",
    ),
    ProductItemModel(
      id: "prod_002",
      title: "Surat Banarasi Georgette Saree",
      description: "Lightweight festive handwoven floral motifs",
      priceInr: 3200.0,
      category: "Textiles",
      sku: "BANARAS-002",
    ),
    ProductItemModel(
      id: "prod_003",
      title: "Chanderi Cotton Embroidered Kurti",
      description: "Breathable pure organic cotton summer collection",
      priceInr: 1200.0,
      category: "Ready-to-wear",
      sku: "CHAND-003",
    ),
  ];

  final Map<String, int> cart = {};

  void _addToCart(String id) {
    setState(() {
      cart[id] = (cart[id] ?? 0) + 1;
    });
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text("Added to Cart"),
        duration: Duration(seconds: 1),
      ),
    );
  }

  void _openCheckoutDialog() {
    double subtotal = 0;
    cart.forEach((id, qty) {
      final p = products.firstWhere((item) => item.id == id);
      subtotal += p.priceInr * qty;
    });

    final gstRate = 5.0; // 5% GST for textiles
    final totalGst = subtotal * (gstRate / 100.0);
    final cgst = totalGst / 2.0;
    final sgst = totalGst / 2.0;
    final grandTotal = subtotal + totalGst;

    showModalBottomSheet(
      context: context,
      backgroundColor: AppTheme.surfaceDark,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(16)),
      ),
      builder: (ctx) {
        return Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                "Order Cart Summary (NPCI UPI 2.0)",
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold, color: AppTheme.textPrimary),
              ),
              const SizedBox(height: 12),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  const Text("Items Subtotal:", style: TextStyle(color: AppTheme.textSecondary)),
                  Text("₹${subtotal.toStringAsFixed(2)}", style: const TextStyle(color: AppTheme.textPrimary)),
                ],
              ),
              const SizedBox(height: 6),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text("CGST (2.5%):", style: const TextStyle(color: AppTheme.textSecondary)),
                  Text("₹${cgst.toStringAsFixed(2)}", style: const TextStyle(color: AppTheme.textPrimary)),
                ],
              ),
              const SizedBox(height: 6),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text("SGST (2.5%):", style: const TextStyle(color: AppTheme.textSecondary)),
                  Text("₹${sgst.toStringAsFixed(2)}", style: const TextStyle(color: AppTheme.textPrimary)),
                ],
              ),
              const Divider(color: Colors.grey, height: 24),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  const Text("Grand Total (INR):", style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold, color: AppTheme.textPrimary)),
                  Text("₹${grandTotal.toStringAsFixed(2)}", style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold, color: AppTheme.primaryGreen)),
                ],
              ),
              const SizedBox(height: 12),
              Row(
                children: const [
                  Icon(Icons.sim_card_outlined, size: 14, color: AppTheme.primaryGreen),
                  SizedBox(width: 6),
                  Text("Hardware SIM-Bound: Pixel 9 Pro (Airtel)", style: TextStyle(fontSize: 11, color: AppTheme.textSecondary)),
                ],
              ),
              const SizedBox(height: 16),
              SizedBox(
                width: double.infinity,
                child: ElevatedButton.icon(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppTheme.primaryGreen,
                    foregroundColor: Colors.white,
                    padding: const EdgeInsets.symmetric(vertical: 14),
                  ),
                  icon: const Icon(Icons.account_balance_wallet_outlined),
                  label: const Text("Pay with UPI (Dynamic Intent)"),
                  onPressed: () {
                    Navigator.pop(ctx);
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(
                        content: Text("Dispatched NPCI UPI URI: upi://pay?pa=suratsilk@okhdfcbank&am=${grandTotal.toStringAsFixed(2)}&cu=INR"),
                        backgroundColor: AppTheme.outboundBubble,
                      ),
                    );
                  },
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final totalItems = cart.values.fold<int>(0, (sum, count) => sum + count);

    return Scaffold(
      appBar: AppBar(
        title: const Text("Surat Heritage Silk Catalog"),
        actions: [
          Stack(
            children: [
              IconButton(
                icon: const Icon(Icons.shopping_bag_outlined),
                onPressed: totalItems > 0 ? _openCheckoutDialog : null,
              ),
              if (totalItems > 0)
                Positioned(
                  right: 8,
                  top: 8,
                  child: CircleAvatar(
                    radius: 8,
                    backgroundColor: AppTheme.primaryGreen,
                    child: Text(
                      "$totalItems",
                      style: const TextStyle(fontSize: 10, color: Colors.white),
                    ),
                  ),
                ),
            ],
          ),
        ],
      ),
      body: ListView.builder(
        padding: const EdgeInsets.all(12),
        itemCount: products.length,
        itemBuilder: (context, index) {
          final p = products[index];
          return Card(
            color: AppTheme.surfaceDark,
            margin: const EdgeInsets.only(bottom: 12),
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Container(
                        width: 80,
                        height: 80,
                        decoration: BoxDecoration(
                          color: AppTheme.darkBackground,
                          borderRadius: BorderRadius.circular(8),
                        ),
                        child: const Icon(Icons.shopping_bag, size: 36, color: AppTheme.textSecondary),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              p.title,
                              style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w600, color: AppTheme.textPrimary),
                            ),
                            const SizedBox(height: 4),
                            Text(
                              p.description,
                              style: const TextStyle(fontSize: 12, color: AppTheme.textSecondary),
                              maxLines: 2,
                              overflow: TextOverflow.ellipsis,
                            ),
                            const SizedBox(height: 6),
                            Text(
                              "₹${p.priceInr.toStringAsFixed(2)}",
                              style: const TextStyle(fontSize: 15, fontWeight: FontWeight.bold, color: AppTheme.primaryGreen),
                            ),
                          ],
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                        decoration: BoxDecoration(
                          color: Colors.black26,
                          borderRadius: BorderRadius.circular(4),
                        ),
                        child: Text(
                          "JSON-LD • SKU: ${p.sku}",
                          style: const TextStyle(fontSize: 10, color: AppTheme.textSecondary),
                        ),
                      ),
                      ElevatedButton.icon(
                        style: ElevatedButton.styleFrom(
                          backgroundColor: AppTheme.primaryGreen,
                          foregroundColor: Colors.white,
                        ),
                        icon: const Icon(Icons.add_shopping_cart, size: 16),
                        label: const Text("Add to Cart"),
                        onPressed: () => _addToCart(p.id),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}
