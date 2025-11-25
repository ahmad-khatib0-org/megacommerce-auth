use phf::{phf_map, Map};

pub(super) static ROUTES: Map<&'static str, bool> = phf_map! {
  "/users.v1.UsersService/CreateSupplier" =>  false,
  "/users.v1.UsersService/Login" =>  false,

  "/products.v1.ProductsService/ProductData" => true,
  "/products.v1.ProductsService/ProductCreate" => true,
  "/products.v1.ProductsService/ProductList" => true,
  "/products.v1.ProductsService/BestSellingProducts" => false,
};
