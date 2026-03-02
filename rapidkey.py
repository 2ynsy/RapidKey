"""
RapidKey – Windows キー連打ツール
依存: pip install keyboard
管理者権限で実行してください (keyboard ライブラリ要件)
"""

import tkinter as tk
from tkinter import ttk, font
import keyboard
import threading
import time
import sys

# ─────────────────────────────────────────────
# テーマカラー
# ─────────────────────────────────────────────
BG        = "#0d0f18"
SURFACE   = "#13162b"
CARD      = "#1a1e36"
BORDER    = "#2a2f52"
ACCENT    = "#6c63ff"
ACCENT2   = "#ff6bfc"
GREEN     = "#39e88f"
RED       = "#ff5e7a"
TEXT      = "#e2e5ff"
MUTED     = "#7880a8"

# ─────────────────────────────────────────────
class RapidKeyApp:
    def __init__(self, root: tk.Tk):
        self.root = root
        self.root.title("RapidKey ⚡ キー連打ツール")
        self.root.configure(bg=BG)
        self.root.resizable(False, False)
        self.root.geometry("480x660")

        # State
        self.target_key   = tk.StringVar(value="")
        self.cps          = tk.IntVar(value=10)
        self.mode         = tk.StringVar(value="toggle")   # toggle / hold / count
        self.repeat_count = tk.IntVar(value=100)
        self.is_running   = False
        self._stop_event  = threading.Event()
        self._thread: threading.Thread | None = None
        self._capturing   = False

        self.total_count  = 0
        self.start_time   = 0.0
        self._recent: list[float] = []

        self._build_ui()
        self._register_hotkey()

        self.root.protocol("WM_DELETE_WINDOW", self._on_close)

    # ─── UI Build ────────────────────────────────
    def _build_ui(self):
        pad = {"padx": 18}

        # Title
        title_frame = tk.Frame(self.root, bg=BG)
        title_frame.pack(fill="x", pady=(22, 0))
        tk.Label(title_frame, text="⚡  RapidKey",
                 bg=BG, fg=TEXT,
                 font=("Segoe UI", 22, "bold")).pack()
        tk.Label(title_frame, text="Windows キー連打ツール",
                 bg=BG, fg=MUTED, font=("Segoe UI", 10)).pack(pady=(2, 0))

        self._sep(18)

        # ── Key capture card ──
        self._card_label("📌  ターゲットキー")
        self.key_btn = tk.Button(
            self.root, textvariable=self.target_key,
            bg=SURFACE, fg=TEXT,
            relief="flat", cursor="hand2",
            font=("Segoe UI", 16, "bold"),
            activebackground=CARD, activeforeground=ACCENT,
            bd=0, height=2,
            command=self._start_capture
        )
        self.key_btn.pack(fill="x", **pad, pady=(0, 6))
        self.target_key.set("← クリックしてキーを設定")

        # Quick-key buttons
        qf = tk.Frame(self.root, bg=BG)
        qf.pack(fill="x", **pad, pady=(0, 0))
        tk.Label(qf, text="よく使う：", bg=BG, fg=MUTED,
                 font=("Segoe UI", 9)).pack(side="left", padx=(0, 6))
        for label, key in [("Space","space"),("Enter","enter"),("F5","f5"),
                            ("Z","z"),("X","x"),("C","c")]:
            b = tk.Button(qf, text=label, bg=SURFACE, fg=TEXT,
                          font=("Segoe UI", 9), relief="flat",
                          cursor="hand2", bd=0, padx=8, pady=3,
                          activebackground=CARD, activeforeground=ACCENT,
                          command=lambda k=key, l=label: self._set_key(k, l))
            b.pack(side="left", padx=2)

        self._sep(14)

        # ── Speed ──
        self._card_label("⚙️  速度")
        sf = tk.Frame(self.root, bg=BG)
        sf.pack(fill="x", **pad, pady=(0, 6))
        tk.Label(sf, text="CPS", bg=BG, fg=MUTED,
                 font=("Segoe UI", 10)).pack(side="left")
        self.cps_lbl = tk.Label(sf, text=f"{self.cps.get()} CPS",
                                 bg=BG, fg=ACCENT,
                                 font=("Segoe UI", 10, "bold"))
        self.cps_lbl.pack(side="right")
        self.slider = tk.Scale(
            self.root, from_=1, to=60,
            orient="horizontal", variable=self.cps,
            bg=BG, fg=TEXT, troughcolor=BORDER,
            activebackground=ACCENT, highlightthickness=0, bd=0,
            sliderrelief="flat", showvalue=False,
            command=self._on_speed_change
        )
        self.slider.pack(fill="x", **pad, pady=(0, 0))

        self._sep(14)

        # ── Mode ──
        self._card_label("🔁  連打モード")
        mf = tk.Frame(self.root, bg=BG)
        mf.pack(fill="x", **pad, pady=(0, 6))
        self._mode_btns = {}
        for label, val in [("🔁 トグル", "toggle"), ("🖱 ホールド", "hold"), ("🔢 回数指定", "count")]:
            b = tk.Button(mf, text=label, bg=SURFACE, fg=MUTED,
                          font=("Segoe UI", 9), relief="flat", cursor="hand2",
                          bd=0, padx=10, pady=5,
                          activebackground=CARD, activeforeground=ACCENT,
                          command=lambda v=val, lb=label: self._set_mode(v))
            b.pack(side="left", padx=(0, 6))
            self._mode_btns[val] = b
        # _set_mode() is called after count_frame is built (see below)

        # Count row
        self.count_frame = tk.Frame(self.root, bg=BG)
        self.count_frame.pack(fill="x", **pad, pady=(4, 0))
        tk.Label(self.count_frame, text="回数：", bg=BG, fg=MUTED,
                 font=("Segoe UI", 10)).pack(side="left")
        tk.Spinbox(self.count_frame, from_=1, to=999999,
                   textvariable=self.repeat_count, width=8,
                   bg=SURFACE, fg=TEXT, buttonbackground=SURFACE,
                   insertbackground=TEXT, relief="flat",
                   font=("Consolas", 11)).pack(side="left", padx=6)
        tk.Label(self.count_frame, text="回", bg=BG, fg=MUTED,
                 font=("Segoe UI", 10)).pack(side="left")
        self.count_frame.pack_forget()
        self._set_mode("toggle")  # safe to call now that count_frame exists

        # Hotkey hint
        hf = tk.Frame(self.root, bg=BG)
        hf.pack(fill="x", **pad, pady=(10, 0))
        tk.Label(hf, text="グローバルホットキー：", bg=BG, fg=MUTED,
                 font=("Segoe UI", 9)).pack(side="left")
        tk.Label(hf, text=" F8 ", bg=SURFACE, fg=TEXT,
                 font=("Consolas", 9), relief="flat", bd=0, padx=4).pack(side="left", padx=4)
        tk.Label(hf, text="で開始 / 停止  |  どのウィンドウがアクティブでも動作", bg=BG, fg=MUTED,
                 font=("Segoe UI", 9)).pack(side="left")

        self._sep(14)

        # ── Main Button ──
        self.main_btn = tk.Button(
            self.root, text="▶  連打 開始",
            bg=ACCENT, fg="white",
            font=("Segoe UI", 14, "bold"),
            relief="flat", cursor="hand2",
            activebackground="#5c54e0", activeforeground="white",
            bd=0, pady=12,
            command=self._on_main_btn
        )
        self.main_btn.pack(fill="x", padx=18, pady=(0, 0))
        self._update_main_btn_state()

        self._sep(14)

        # ── Stats ──
        self._card_label("📊  統計")
        sf2 = tk.Frame(self.root, bg=BG)
        sf2.pack(fill="x", **pad, pady=(0, 6))
        sf2.columnconfigure((0, 1, 2), weight=1)

        self.lbl_total = self._stat_box(sf2, "0",       "TOTAL PRESSES", 0)
        self.lbl_cps   = self._stat_box(sf2, "0",       "実測 CPS",       1)
        self.lbl_time  = self._stat_box(sf2, "0.0 s",   "経過時間",        2)

        # Status bar
        self.status_var = tk.StringVar(value="待機中")
        tk.Label(self.root, textvariable=self.status_var,
                 bg=SURFACE, fg=MUTED,
                 font=("Segoe UI", 9), pady=6).pack(fill="x", side="bottom")

    def _sep(self, pady=10):
        tk.Frame(self.root, bg=BORDER, height=1).pack(fill="x", padx=18, pady=pady)

    def _card_label(self, text):
        tk.Label(self.root, text=text, bg=BG, fg=MUTED,
                 font=("Segoe UI", 9, "bold"),
                 anchor="w").pack(fill="x", padx=18, pady=(0, 6))

    def _stat_box(self, parent, val, label, col):
        f = tk.Frame(parent, bg=CARD, pady=10, padx=6)
        f.grid(row=0, column=col, padx=4, sticky="nsew")
        v = tk.Label(f, text=val, bg=CARD, fg=TEXT,
                     font=("Consolas", 18, "bold"))
        v.pack()
        tk.Label(f, text=label, bg=CARD, fg=MUTED,
                 font=("Segoe UI", 8)).pack()
        return v

    # ─── Key Capture ─────────────────────────────
    def _start_capture(self):
        if self._capturing:
            return
        self._capturing = True
        self.key_btn.config(fg=ACCENT, bg=CARD)
        self.target_key.set("▶ キーを押してください…")
        self.root.bind("<KeyPress>", self._on_key_press)
        self.root.focus_force()

    def _on_key_press(self, event: tk.Event):
        if not self._capturing:
            return
        self._capturing = False
        self.root.unbind("<KeyPress>")
        key = event.keysym.lower()
        # Normalize
        if key == "space":   key = "space"
        elif len(key) == 1:  pass
        else:                key = key
        self._set_key(key, event.keysym)

    def _set_key(self, key: str, label: str):
        self._capturing = False
        self.root.unbind("<KeyPress>")
        self._raw_key = key
        self.key_btn.config(fg=TEXT, bg=SURFACE)
        disp = label if len(label) <= 12 else label[:12]
        self.target_key.set(f"  {disp.upper()}  ")
        self._update_main_btn_state()

    # ─── Mode ────────────────────────────────────
    def _set_mode(self, mode: str):
        self.mode.set(mode)
        for k, b in self._mode_btns.items():
            if k == mode:
                b.config(bg=ACCENT, fg="white")
            else:
                b.config(bg=SURFACE, fg=MUTED)
        if hasattr(self, "count_frame"):
            if mode == "count":
                self.count_frame.pack(fill="x", padx=18, pady=(4, 0))
            else:
                self.count_frame.pack_forget()
        if hasattr(self, "main_btn"):
            self._update_main_btn_state()

    # ─── Speed ───────────────────────────────────
    def _on_speed_change(self, _=None):
        self.cps_lbl.config(text=f"{self.cps.get()} CPS")

    # ─── Main Button ─────────────────────────────
    def _update_main_btn_state(self):
        if not hasattr(self, "main_btn"):
            return
        has_key = hasattr(self, "_raw_key") and self._raw_key
        if not has_key:
            self.main_btn.config(state="disabled", bg=BORDER, fg=MUTED,
                                  text="▶  連打 開始（キーを設定してください）")
        elif self.is_running:
            self.main_btn.config(state="normal", bg=RED, fg="white",
                                  text="⬛  停止")
        else:
            self.main_btn.config(state="normal", bg=ACCENT, fg="white",
                                  text="▶  連打 開始")

    def _on_main_btn(self):
        mode = self.mode.get()
        if mode == "hold":
            return  # hold handled separately (not implemented with tkinter button easily; use F8)
        self._toggle()

    def _toggle(self):
        if self.is_running:
            self._stop()
        else:
            self._start()

    # ─── Start / Stop ────────────────────────────
    def _start(self):
        if self.is_running or not hasattr(self, "_raw_key"):
            return
        self.is_running = True
        self._stop_event.clear()
        self.total_count = 0
        self.start_time  = time.perf_counter()
        self._recent.clear()
        self._update_main_btn_state()
        self.status_var.set(f"🟢 連打中: {self._raw_key.upper()}  @{self.cps.get()} CPS")

        limit = self.repeat_count.get() if self.mode.get() == "count" else None
        self._thread = threading.Thread(target=self._fire_loop,
                                        args=(limit,), daemon=True)
        self._thread.start()
        self._tick_stats()

    def _stop(self):
        self.is_running = False
        self._stop_event.set()
        self.status_var.set("⏸ 停止")
        self._update_main_btn_state()

    def _fire_loop(self, limit):
        interval = 1.0 / max(1, self.cps.get())
        fired = 0
        next_time = time.perf_counter()

        while not self._stop_event.is_set():
            keyboard.press_and_release(self._raw_key)
            fired += 1
            self.total_count += 1
            self._recent.append(time.perf_counter())

            if limit and fired >= limit:
                self.root.after(0, self._stop)
                break

            next_time += interval
            sleep_dur = next_time - time.perf_counter()
            if sleep_dur > 0:
                time.sleep(sleep_dur)

    def _tick_stats(self):
        if not self.is_running and self.total_count == 0:
            return
        now = time.perf_counter()
        elapsed = now - self.start_time if self.start_time > 0 else 0

        # real CPS over last second
        self._recent = [t for t in self._recent if now - t < 1.0]
        real_cps = len(self._recent)

        self.lbl_total.config(text=f"{self.total_count:,}", fg=GREEN if self.is_running else TEXT)
        self.lbl_cps.config(text=str(real_cps),             fg=GREEN if self.is_running else TEXT)
        self.lbl_time.config(text=f"{elapsed:.1f} s",       fg=TEXT)

        if self.is_running:
            self.root.after(120, self._tick_stats)

    # ─── Hotkey F8 ───────────────────────────────
    def _register_hotkey(self):
        try:
            keyboard.add_hotkey("f8", self._on_f8, suppress=True)
        except Exception as e:
            self.status_var.set(f"⚠ F8登録失敗: {e}")

    def _on_f8(self):
        """Called from background thread → schedule on main thread."""
        self.root.after(0, self._toggle)

    # ─── Close ───────────────────────────────────
    def _on_close(self):
        self._stop()
        try:
            keyboard.unhook_all()
        except Exception:
            pass
        self.root.destroy()


# ─────────────────────────────────────────────
if __name__ == "__main__":
    root = tk.Tk()
    app  = RapidKeyApp(root)
    root.mainloop()
