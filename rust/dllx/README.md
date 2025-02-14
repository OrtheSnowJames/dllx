# DLLX
This project is licensed under the AGPL v3.0 (Affero General Public License Version 3.0). See the [LICENSE](./LICENSE) file for more details.

Dllx is an effort made to unite linking libraries for all platforms. It is a zip archive with all kinds of linking libraries,
like .so, .dylib and .dll. So far we have rust and go support, but we don't have any other language support. If you wish to
contribute and bring one of your languages to it, you are more than welcome to contribute, but your pull request might not
get approved based on what you do in it.

## To use in rust:

add it
```bash
cargo add dllx
```

import it
```rust
use dllx::load_and_call;
```

and use it
```rust
fn main() {
    match load_and_call("your_file.dllx", "Foo") {
        Ok(_) => println!("Successfully called function 'Foo'!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## To use in golang (WARN: Does not support Windows as go-Windows does not have the plugin stl library, WARN 2: This crate does not have the go mod):

first, get the repo.
```bash
go get github.com/OrtheSnowJames/dllx/go/dllx
```
import it
```go
import (
    // other stuff
    // you don't need to import everything here it's just an example of a project imports
	"archive/zip"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"plugin"
	"runtime"
    "github.com/OrtheSnowJames/dllx/go/dllx"
)
```
and use it
```go
func main() {
	// Path to the .dllx file (replace with your actual file path)
	dllxFile := "your_file.dllx"

	// Read manifest from the .dllx file
	manifest, err := readManifestFromDllx(dllxFile)
	if err != nil {
		fmt.Printf("Error reading manifest: %v\n", err)
		return
	}

	// Load the platform-specific library dynamically
	plug, err := loadLibrary(dllxFile, manifest)
	if err != nil {
		fmt.Printf("Error loading library: %v\n", err)
		return
	}

	// Assume the shared library exposes a function called 'Foo'
	fooSymbol, err := plug.Lookup("Foo")
	if err != nil {
		fmt.Printf("Error looking up symbol 'Foo': %v\n", err)
		return
	}

	// Call the function (assuming it takes no arguments and returns no values)
	fooFunc := fooSymbol.(func())
	fooFunc()

	fmt.Println("Successfully called Foo from the dynamic library!")
}
```