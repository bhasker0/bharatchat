import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/theme/app_theme.dart';
import '../auth/auth_state.dart';
import 'catalog_state.dart';

class CatalogScreen extends ConsumerStatefulWidget {
  const CatalogScreen({super.key});

  @override
  ConsumerState<CatalogScreen> createState() => _CatalogScreenState();
}

class _CatalogScreenState extends ConsumerState<CatalogScreen> {
  String _selectedCategory = "All";

  void _showAddProductDialog(String ownerDid, String ownerName) {
    final titleController = TextEditingController();
    final descController = TextEditingController();
    final priceController = TextEditingController();
    final categoryController = TextEditingController(text: "Textiles");
    final skuController = TextEditingController(text: "TEX-${DateTime.now().millisecondsSinceEpoch % 10000}");

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppTheme.surfaceDark,
        title: const Text("Add Product to Your Shop", style: TextStyle(color: AppTheme.textPrimary)),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: titleController,
                style: const TextStyle(color: AppTheme.textPrimary),
                decoration: const InputDecoration(labelText: "Product Title *", border: OutlineInputBorder()),
              ),
              const SizedBox(height: 10),
              TextField(
                controller: descController,
                style: const TextStyle(color: AppTheme.textPrimary),
                decoration: const InputDecoration(labelText: "Description", border: OutlineInputBorder()),
                maxLines: 2,
              ),
              const SizedBox(height: 10),
              TextField(
                controller: priceController,
                style: const TextStyle(color: AppTheme.textPrimary),
                keyboardType: const TextInputType.numberWithOptions(decimal: true),
                decoration: const InputDecoration(labelText: "Price (INR) *", prefixText: "₹ ", border: OutlineInputBorder()),
              ),
              const SizedBox(height: 10),
              TextField(
                controller: categoryController,
                style: const TextStyle(color: AppTheme.textPrimary),
                decoration: const InputDecoration(labelText: "Category", border: OutlineInputBorder()),
              ),
              const SizedBox(height: 10),
              TextField(
                controller: skuController,
                style: const TextStyle(color: AppTheme.textPrimary),
                decoration: const InputDecoration(labelText: "SKU / Code", border: OutlineInputBorder()),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text("Cancel", style: TextStyle(color: AppTheme.textSecondary)),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: AppTheme.primaryGreen),
            onPressed: () async {
              final title = titleController.text.trim();
              final price = double.tryParse(priceController.text.trim()) ?? 0.0;
              if (title.isNotEmpty && price > 0) {
                final scaffoldMessenger = ScaffoldMessenger.of(context);
                Navigator.pop(ctx);
                await ref.read(catalogProvider.notifier).addProduct(
                      ownerDid: ownerDid,
                      ownerName: ownerName,
                      title: title,
                      description: descController.text.trim(),
                      priceInr: price,
                      category: categoryController.text.trim(),
                      sku: skuController.text.trim(),
                    );
                if (mounted) {
                  scaffoldMessenger.showSnackBar(
                    const SnackBar(content: Text("Product listed in your sovereign shop!")),
                  );
                }
              }
            },
            child: const Text("List Product", style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );
  }

  void _showEditProductDialog(ProductItemModel product) {
    final titleController = TextEditingController(text: product.title);
    final descController = TextEditingController(text: product.description);
    final priceController = TextEditingController(text: product.priceInr.toStringAsFixed(0));
    final categoryController = TextEditingController(text: product.category);

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppTheme.surfaceDark,
        title: const Text("Edit Your Product", style: TextStyle(color: AppTheme.textPrimary)),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: titleController,
                style: const TextStyle(color: AppTheme.textPrimary),
                decoration: const InputDecoration(labelText: "Product Title", border: OutlineInputBorder()),
              ),
              const SizedBox(height: 10),
              TextField(
                controller: descController,
                style: const TextStyle(color: AppTheme.textPrimary),
                decoration: const InputDecoration(labelText: "Description", border: OutlineInputBorder()),
                maxLines: 2,
              ),
              const SizedBox(height: 10),
              TextField(
                controller: priceController,
                style: const TextStyle(color: AppTheme.textPrimary),
                keyboardType: const TextInputType.numberWithOptions(decimal: true),
                decoration: const InputDecoration(labelText: "Price (INR)", prefixText: "₹ ", border: OutlineInputBorder()),
              ),
              const SizedBox(height: 10),
              TextField(
                controller: categoryController,
                style: const TextStyle(color: AppTheme.textPrimary),
                decoration: const InputDecoration(labelText: "Category", border: OutlineInputBorder()),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text("Cancel", style: TextStyle(color: AppTheme.textSecondary)),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: AppTheme.primaryGreen),
            onPressed: () async {
              final title = titleController.text.trim();
              final price = double.tryParse(priceController.text.trim()) ?? product.priceInr;
              final scaffoldMessenger = ScaffoldMessenger.of(context);
              Navigator.pop(ctx);
              await ref.read(catalogProvider.notifier).updateProduct(
                    id: product.id,
                    title: title.isNotEmpty ? title : product.title,
                    description: descController.text.trim(),
                    priceInr: price,
                    category: categoryController.text.trim(),
                  );
              if (mounted) {
                scaffoldMessenger.showSnackBar(
                  const SnackBar(content: Text("Product updated successfully!")),
                );
              }
            },
            child: const Text("Save Changes", style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );
  }

  void _showDeleteConfirmation(ProductItemModel product) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppTheme.surfaceDark,
        title: const Text("Delete Product?", style: TextStyle(color: Colors.redAccent)),
        content: Text("Are you sure you want to remove '${product.title}' from your shop?",
            style: const TextStyle(color: AppTheme.textSecondary)),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text("Cancel", style: TextStyle(color: AppTheme.textSecondary)),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: Colors.redAccent),
            onPressed: () async {
              final scaffoldMessenger = ScaffoldMessenger.of(context);
              Navigator.pop(ctx);
              await ref.read(catalogProvider.notifier).deleteProduct(product.id);
              if (mounted) {
                scaffoldMessenger.showSnackBar(
                  const SnackBar(content: Text("Product removed from catalog.")),
                );
              }
            },
            child: const Text("Delete", style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );
  }

  void _openCheckoutDialog(CatalogState catalogState) {
    double subtotal = 0;
    catalogState.cart.forEach((id, qty) {
      final p = catalogState.products.firstWhere((item) => item.id == id,
          orElse: () => const ProductItemModel(
                id: '',
                ownerDid: '',
                ownerName: '',
                title: 'Item',
                description: '',
                priceInr: 0,
                category: '',
                sku: '',
              ));
      subtotal += p.priceInr * qty;
    });

    const gstRate = 5.0; // 5% GST for textiles
    final totalGst = subtotal * (gstRate / 100.0);
    final grandTotal = subtotal + totalGst;

    showModalBottomSheet(
      context: context,
      backgroundColor: AppTheme.surfaceDark,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (context) {
        return Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Row(
                children: [
                  Icon(Icons.receipt_long, color: AppTheme.primaryGreen),
                  SizedBox(width: 8),
                  Text("Tax Invoice & UPI Intent", style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
                ],
              ),
              const Divider(color: AppTheme.darkBackground),
              Text("Subtotal (${catalogState.cart.values.fold(0, (a, b) => a + b)} items): ₹${subtotal.toStringAsFixed(2)}"),
              Text("GST @ 5% (SAC 9988): ₹${totalGst.toStringAsFixed(2)}", style: const TextStyle(color: AppTheme.textSecondary, fontSize: 12)),
              const SizedBox(height: 8),
              Text(
                "Grand Total: ₹${grandTotal.toStringAsFixed(2)}",
                style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold, color: AppTheme.primaryGreen),
              ),
              const SizedBox(height: 16),
              SizedBox(
                width: double.infinity,
                child: ElevatedButton.icon(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppTheme.primaryGreen,
                    padding: const EdgeInsets.symmetric(vertical: 14),
                  ),
                  icon: const Icon(Icons.qr_code_scanner, color: Colors.white),
                  label: const Text("Pay via Bharat UPI", style: TextStyle(color: Colors.white, fontSize: 16)),
                  onPressed: () {
                    Navigator.pop(context);
                    ref.read(catalogProvider.notifier).clearCart();
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(
                        content: Text("Simulated UPI Intent triggered for ₹${grandTotal.toStringAsFixed(2)}"),
                        backgroundColor: AppTheme.primaryGreen,
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
    final currentUser = ref.watch(authProvider).user;
    final currentUserDid = currentUser?.did ?? '';
    final currentUserName = currentUser?.name ?? 'Sovereign Merchant';
    final catalogState = ref.watch(catalogProvider);

    final categories = ["All", ...{...catalogState.products.map((p) => p.category)}];
    final filtered = _selectedCategory == "All"
        ? catalogState.products
        : catalogState.products.where((p) => p.category == _selectedCategory).toList();

    final totalCartItems = catalogState.cart.values.fold(0, (a, b) => a + b);

    return Scaffold(
      appBar: AppBar(
        title: const Text("Sovereign Catalog"),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: "Refresh Catalog",
            onPressed: () => ref.read(catalogProvider.notifier).loadProducts(),
          ),
          Stack(
            alignment: Alignment.center,
            children: [
              IconButton(
                icon: const Icon(Icons.shopping_cart_outlined),
                onPressed: totalCartItems > 0 ? () => _openCheckoutDialog(catalogState) : null,
              ),
              if (totalCartItems > 0)
                Positioned(
                  right: 8,
                  top: 8,
                  child: Container(
                    padding: const EdgeInsets.all(4),
                    decoration: const BoxDecoration(color: AppTheme.primaryGreen, shape: BoxShape.circle),
                    child: Text(
                      "$totalCartItems",
                      style: const TextStyle(color: Colors.white, fontSize: 10, fontWeight: FontWeight.bold),
                    ),
                  ),
                ),
            ],
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        heroTag: 'catalog_add_fab',
        backgroundColor: AppTheme.primaryGreen,
        foregroundColor: Colors.white,
        icon: const Icon(Icons.add_business),
        label: const Text("Add Product"),
        onPressed: () => _showAddProductDialog(currentUserDid, currentUserName),
      ),
      body: Column(
        children: [
          // Category pills
          SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Row(
              children: categories.map((cat) {
                final isSelected = cat == _selectedCategory;
                return Padding(
                  padding: const EdgeInsets.only(right: 8.0),
                  child: ChoiceChip(
                    label: Text(cat),
                    selected: isSelected,
                    selectedColor: AppTheme.primaryGreen,
                    labelStyle: TextStyle(
                      color: isSelected ? Colors.white : AppTheme.textSecondary,
                      fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                    ),
                    backgroundColor: AppTheme.surfaceDark,
                    onSelected: (val) {
                      if (val) setState(() => _selectedCategory = cat);
                    },
                  ),
                );
              }).toList(),
            ),
          ),

          // Product List
          Expanded(
            child: catalogState.isLoading
                ? const Center(child: CircularProgressIndicator())
                : filtered.isEmpty
                    ? Center(
                        child: Column(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            const Icon(Icons.storefront_outlined, size: 64, color: AppTheme.textSecondary),
                            const SizedBox(height: 12),
                            const Text("No products found in this category", style: TextStyle(color: AppTheme.textSecondary)),
                            const SizedBox(height: 8),
                            ElevatedButton.icon(
                              style: ElevatedButton.styleFrom(backgroundColor: AppTheme.primaryGreen),
                              icon: const Icon(Icons.add, color: Colors.white),
                              label: const Text("List First Product", style: TextStyle(color: Colors.white)),
                              onPressed: () => _showAddProductDialog(currentUserDid, currentUserName),
                            ),
                          ],
                        ),
                      )
                    : ListView.builder(
                        padding: const EdgeInsets.fromLTRB(16, 8, 16, 80),
                        itemCount: filtered.length,
                        itemBuilder: (context, index) {
                          final item = filtered[index];
                          final isOwner = currentUserDid.isNotEmpty && item.ownerDid == currentUserDid;

                          return Card(
                            color: AppTheme.surfaceDark,
                            margin: const EdgeInsets.symmetric(vertical: 8),
                            shape: RoundedRectangleBorder(
                              borderRadius: BorderRadius.circular(12),
                              side: BorderSide(
                                color: isOwner ? AppTheme.primaryGreen.withOpacity(0.5) : Colors.transparent,
                              ),
                            ),
                            child: Padding(
                              padding: const EdgeInsets.all(16.0),
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  Row(
                                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                                    children: [
                                      Expanded(
                                        child: Text(
                                          item.title,
                                          style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
                                        ),
                                      ),
                                      Text(
                                        "₹${item.priceInr.toStringAsFixed(0)}",
                                        style: const TextStyle(
                                          fontSize: 16,
                                          fontWeight: FontWeight.bold,
                                          color: AppTheme.primaryGreen,
                                        ),
                                      ),
                                    ],
                                  ),
                                  const SizedBox(height: 4),
                                  Text(
                                    item.description,
                                    style: const TextStyle(color: AppTheme.textSecondary, fontSize: 13),
                                  ),
                                  const SizedBox(height: 8),
                                  Row(
                                    children: [
                                      Container(
                                        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                                        decoration: BoxDecoration(
                                          color: AppTheme.darkBackground,
                                          borderRadius: BorderRadius.circular(6),
                                        ),
                                        child: Text(item.category, style: const TextStyle(fontSize: 11, color: AppTheme.accentBlue)),
                                      ),
                                      const SizedBox(width: 8),
                                      Text("SKU: ${item.sku}", style: const TextStyle(fontSize: 11, color: AppTheme.textSecondary)),
                                      const Spacer(),
                                      if (isOwner)
                                        Container(
                                          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                                          decoration: BoxDecoration(
                                            color: AppTheme.primaryGreen.withOpacity(0.2),
                                            borderRadius: BorderRadius.circular(6),
                                          ),
                                          child: const Row(
                                            children: [
                                              Icon(Icons.store, size: 12, color: AppTheme.primaryGreen),
                                              SizedBox(width: 4),
                                              Text("Your Listing", style: TextStyle(fontSize: 11, color: AppTheme.primaryGreen, fontWeight: FontWeight.bold)),
                                            ],
                                          ),
                                        )
                                      else
                                        Text(
                                          "Sold by: ${item.ownerName}",
                                          style: const TextStyle(fontSize: 11, color: AppTheme.textSecondary),
                                        ),
                                    ],
                                  ),
                                  const SizedBox(height: 12),
                                  Row(
                                    mainAxisAlignment: MainAxisAlignment.end,
                                    children: [
                                      if (isOwner) ...[
                                        // Owner controls: EDIT and DELETE (NO "Add to Cart")
                                        OutlinedButton.icon(
                                          icon: const Icon(Icons.edit, size: 16, color: AppTheme.primaryGreen),
                                          label: const Text("Edit", style: TextStyle(color: AppTheme.primaryGreen)),
                                          onPressed: () => _showEditProductDialog(item),
                                        ),
                                        const SizedBox(width: 8),
                                        IconButton(
                                          icon: const Icon(Icons.delete_outline, color: Colors.redAccent),
                                          tooltip: "Delete Product",
                                          onPressed: () => _showDeleteConfirmation(item),
                                        ),
                                      ] else ...[
                                        // Customer controls: "Add to Cart" and "Buy via UPI"
                                        ElevatedButton.icon(
                                          style: ElevatedButton.styleFrom(
                                            backgroundColor: AppTheme.primaryGreen,
                                            foregroundColor: Colors.white,
                                          ),
                                          icon: const Icon(Icons.add_shopping_cart, size: 18),
                                          label: const Text("Add to Cart"),
                                          onPressed: () {
                                            ref.read(catalogProvider.notifier).addToCart(item.id);
                                            ScaffoldMessenger.of(context).showSnackBar(
                                              SnackBar(
                                                content: Text("Added '${item.title}' to cart!"),
                                                duration: const Duration(seconds: 1),
                                              ),
                                            );
                                          },
                                        ),
                                        const SizedBox(width: 8),
                                        OutlinedButton.icon(
                                          icon: const Icon(Icons.payment, size: 18, color: AppTheme.accentBlue),
                                          label: const Text("Buy via UPI", style: TextStyle(color: AppTheme.accentBlue)),
                                          onPressed: () {
                                            ref.read(catalogProvider.notifier).addToCart(item.id);
                                            _openCheckoutDialog(catalogState);
                                          },
                                        ),
                                      ],
                                    ],
                                  ),
                                ],
                              ),
                            ),
                          );
                        },
                      ),
          ),
        ],
      ),
    );
  }
}
