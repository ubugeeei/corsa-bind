package corsautils

import "testing"

func TestUtilsBindings(t *testing.T) {
	if got := ClassifyTypeText("Promise<string> | null"); got != "nullish" {
		t.Fatalf("classify = %q", got)
	}
	split := SplitTypeText("string | Promise<any>")
	if len(split) != 2 || split[1] != "Promise<any>" {
		t.Fatalf("split = %#v", split)
	}
	if !IsErrorLikeTypeTexts([]string{"TypeError"}, nil) {
		t.Fatal("expected error-like detection")
	}
	if !HasUnsafeAnyFlow([]string{"Promise<any>"}, []string{"Promise<string>"}) {
		t.Fatal("expected unsafe any flow detection")
	}
}

func TestVirtualDocumentBindings(t *testing.T) {
	document, err := NewUntitledVirtualDocument("/demo.ts", "typescript", "const value = 1;")
	if err != nil {
		t.Fatalf("new untitled: %v", err)
	}
	defer document.Close()
	if err := document.Splice(0, 14, 0, 15, "2"); err != nil {
		t.Fatalf("splice: %v", err)
	}
	if got := document.Text(); got != "const value = 2;" {
		t.Fatalf("text = %q", got)
	}
	if got := document.Version(); got != 2 {
		t.Fatalf("version = %d", got)
	}
}
