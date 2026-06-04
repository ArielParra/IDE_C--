# using GNU make:
ifdef OS  #Windows
	FixPath = $(subst /,\,$1)
	InstallPath := $(APPDATA)/Local/Programs/IDE_C--/
	DesktopPath := $(USERPROFILE)/Desktop/
	UNAME := Windows
	EXT := .exe
else  	  #*NIX
	FixPath = $1
	InstallPath := $(HOME)/.local/bin/
	DesktopPath := $(HOME)/.local/share/applications/
	IconPath := $(HOME)/.local/share/icons/hicolor/
	UNAME = $(shell uname)
	EXT :=
endif

Name := IDE_C--
DesktopFile := com.ide_cmm.ide.desktop
DesktopSource := src/resources/$(DesktopFile)
IconSourceDir := src/resources/icons/hicolor

all: build

build:
	cargo build --release

install: build
ifeq ($(UNAME), Linux)
	mkdir -p "$(call FixPath,$(InstallPath))"
	cp $(call FixPath,target/release/$(Name)$(EXT)) "$(call FixPath,$(InstallPath))"
	
	mkdir -p "$(call FixPath,$(DesktopPath))"
	cp $(call FixPath,$(DesktopSource)) "$(call FixPath,$(DesktopPath))"
	sed -i 's|Exec=.*|Exec=$(InstallPath)$(Name)|' "$(call FixPath,$(DesktopPath)$(DesktopFile))"
	
	mkdir -p "$(call FixPath,$(IconPath))"
	cp -r $(call FixPath,$(IconSourceDir))/* "$(call FixPath,$(IconPath))"
	gtk-update-icon-cache -f "$(call FixPath,$(IconPath))" 2>/dev/null || true
	@echo "Installation complete. You can now launch IDE C--."
else ifeq ($(UNAME), Windows)
	mkdir -p "$(call FixPath,$(InstallPath))"
	cp $(call FixPath,target/release/$(Name)$(EXT)) "$(call FixPath,$(InstallPath))"
	@powershell.exe -Command " \
		$$ws = New-Object -ComObject WScript.Shell; \
		$$shortcut = $$ws.CreateShortcut('$(call FixPath,$(DesktopPath))\\$(Name).lnk'); \
		$$shortcut.TargetPath = '$(call FixPath,$(InstallPath))$(Name)$(EXT)'; \
		$$shortcut.WorkingDirectory = '$(call FixPath,$(InstallPath))'; \
		$$shortcut.Save(); \
	"
	@echo "Installation complete."
endif

uninstall:
ifeq ($(UNAME), Linux)
	rm -f "$(call FixPath,$(InstallPath))$(Name)$(EXT)"
	rm -f "$(call FixPath,$(DesktopPath))$(DesktopFile)"
	find "$(call FixPath,$(IconPath))" -name "com.ide_cmm.ide.*" -type f -delete
	gtk-update-icon-cache -f "$(call FixPath,$(IconPath))" 2>/dev/null || true
	@echo "Uninstallation complete."
else ifeq ($(UNAME), Windows)
	rm -f "$(call FixPath,$(InstallPath))$(Name)$(EXT)"
	rm -f "$(call FixPath,$(DesktopPath))$(Name).lnk"
	@echo "Uninstallation complete."
endif

clean:
	cargo clean

.PHONY: all build install uninstall clean
