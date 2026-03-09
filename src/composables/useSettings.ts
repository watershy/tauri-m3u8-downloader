import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { downloadDir } from '@tauri-apps/api/path';

// Define the shape of your settings object
export interface AppSettings {
    download_path: string;
    play_completion_sound: boolean;
}

export function useSettings() {
    // Initialize with safe defaults
    const settings = ref<AppSettings>({
        download_path: '',
        play_completion_sound: true
    });
    
    const loading = ref(false);

    // Loads settings from Rust.
    // If download_path is missing, it auto-detects the OS default.
    const loadSettings = async () => {
        loading.value = true;
        try {
            const saved = await invoke<AppSettings>('load_settings');

            // Auto-fill default path if missing
            if (!saved.download_path) {
                console.log("No path saved, fetching OS default...");
                saved.download_path = await downloadDir();
            }

            settings.value = saved;
        } catch (e) {
            console.error("Failed to load settings:", e);
        } finally {
            loading.value = false;
        }
    };

    // Persist current settings to disk via Rust
    const saveSettings = async () => {
        try {
            await invoke('save_settings', { settings: settings.value });
            console.log("Settings saved!");
        } catch (e) {
            console.error("Failed to save settings:", e);
        }
    };

    // Opens the OS folder picker and updates settings
    const browseDownloadFolder = async () => {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                defaultPath: settings.value.download_path || await downloadDir()
            });

            if (selected && typeof selected === 'string') {
                settings.value.download_path = selected;
                // Auto-save immediately after selection for better UX
                await saveSettings(); 
            }
        } catch (err) {
            console.error("Failed to open dialog:", err);
        }
    };

    return {
        settings,
        loading,
        loadSettings,
        saveSettings,
        browseDownloadFolder
    };
}