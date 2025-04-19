import customtkinter as ctk
from tkinterdnd2 import DND_FILES, TkinterDnD


# Combined CTk + DnD window
class CTkDnDApp(ctk.CTk, TkinterDnD.DnDWrapper):
    def __init__(self, *args, **kwargs):
        ctk.CTk.__init__(self, *args, **kwargs)
        TkinterDnD.DnDWrapper.__init__(self)
        self.TkdndVersion = TkinterDnD._require(self)


class DragAndDropApp(CTkDnDApp):
    def __init__(self):
        super().__init__()
        self.title("Excel File Sorter")
        self.geometry("500x300")
        ctk.set_appearance_mode("System")
        ctk.set_default_color_theme("blue")

        self.default_fg_color = "gray20"
        self.hover_fg_color = "gray30"
        self.success_fg_color = "green"

        # Drop zone
        self.drop_frame = ctk.CTkFrame(self, width=400, height=120, corner_radius=10)
        self.drop_frame.pack(expand=True)
        self.drop_frame.pack_propagate(False)

        self.drop_label = ctk.CTkLabel(
            self.drop_frame,
            text="📂 Drag and drop your Excel file here\nor click to browse",
            fg_color=self.default_fg_color,
            corner_radius=10,
            justify="center"
        )
        self.drop_label.pack(expand=True, fill="both", padx=10, pady=10)

        self.drop_label.drop_target_register(DND_FILES)
        self.drop_label.dnd_bind("<<Drop>>", self.on_file_drop)

        # Hover effects using mouse events instead of drag events
        self.drop_label.bind("<Enter>", self.on_hover_enter)
        self.drop_label.bind("<Leave>", self.on_hover_leave)

        # Click-to-select
        self.drop_label.bind("<Button-1>", self.open_file_dialog)

    def on_hover_enter(self, event):
        if "File selected" not in self.drop_label.cget("text"):
            self.drop_label.configure(fg_color=self.hover_fg_color)

    def on_hover_leave(self, event):
        if "File selected" not in self.drop_label.cget("text"):
            self.drop_label.configure(fg_color=self.default_fg_color)

    def on_file_drop(self, event):
        file_path = event.data.strip("{}")  # Clean path format
        print(f"[LOG] Dropped file: {file_path}")
        self.drop_label.configure(text=f"✅ File selected:\n{file_path}", fg_color=self.success_fg_color)

    def open_file_dialog(self, event=None):
        from tkinter import filedialog
        # file_path = filedialog.askopenfilename(filetypes=[("Excel files", "*.xlsx *.xls")])
        file_path = filedialog.askopenfilename()
        if file_path:
            print(f"[LOG] Click-selected file: {file_path}")
            self.drop_label.configure(text=f"✅ File selected:\n{file_path}", fg_color=self.success_fg_color)


if __name__ == "__main__":
    app = DragAndDropApp()
    app.mainloop()
