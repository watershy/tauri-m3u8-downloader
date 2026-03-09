import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';

export interface Resolution {
    label: string;
    value: any;
}

export function useNewJob() {
    // --- Form State ---
    const videoUrl = ref('');
    const headers = ref([{ name: 'Referer', value: '' }]);

    const headerOptions = ref([
        'Referer',
        'Authorization',
        'Cookie',
        'Origin',
    ]);

    const saveFilename = ref('');
    const fileExtension = ref('mp4');
    const saveFolder = ref(''); // We store the specific folder for this job here

    // --- Media Check State ---
    const mediaChecked = ref(false);
    const loadingMedia = ref(false);
    const loadingDownload = ref(false);
    const errorMessage = ref('');
    const hasResolution = ref(false);
    const resolutions = ref<Resolution[]>([]);
    const selectedResolution = ref<any>(null);
    const totalSegments = ref(0);

    const extensionOptions = ref([
        { label: 'MP4', value: 'mp4' },
        { label: 'MKV', value: 'mkv' },
        { label: 'MOV', value: 'mov' }
    ]);

    // --- Actions ---

    const addHeader = () => {
        headers.value.push({ name: '', value: '' });
    };

    const removeHeader = (index: number) => {
        headers.value.splice(index, 1);
    };

    const resetForm = (defaultFolder: string) => {
        videoUrl.value = '';
        errorMessage.value = '';
        mediaChecked.value = false;
        resolutions.value = [];
        selectedResolution.value = null;
        saveFilename.value = '';
        fileExtension.value = 'mp4';
        headers.value = [{ name: 'Referer', value: '' }];
        totalSegments.value = 0;
        saveFolder.value = defaultFolder;
    };

    const checkMediaMetadata = async () => {
        errorMessage.value = "";
        mediaChecked.value = false;
        loadingMedia.value = true;
        totalSegments.value = 0;

        const httpHeaders: Record<string, string> = {};
        for (const h of headers.value) {
            if (h.name.trim() && h.value.trim()) {
                httpHeaders[h.name.trim()] = h.value.trim();
            }
        }

        try {
            console.log("Checking media:", videoUrl.value);
            const result = await invoke<any>('check_media', {
                videoUrl: videoUrl.value,
                httpHeaders: httpHeaders
            });

            if (result.success) {
                // Update save location if backend suggests one
                if (result.save_folder) {
                    saveFolder.value = result.save_folder;
                }

                totalSegments.value = result.total_segments || 0;
                
                // Handle Filename
                const fullSuggestedName = result.suggested_filename || "video.mp4";
                const lastDotIndex = fullSuggestedName.lastIndexOf('.');
                if (lastDotIndex > 0) {
                    saveFilename.value = fullSuggestedName.substring(0, lastDotIndex);
                } else {
                    saveFilename.value = fullSuggestedName;
                }

                // Handle Resolutions
                if (result.resolutions && result.resolutions.length > 0) {
                    hasResolution.value = true;
                    resolutions.value = result.resolutions.map((variant: any) => {
                        const labelText = variant.codecs 
                            ? `${variant.resolution || "Unknown"} (${variant.codecs})` 
                            : `${variant.resolution || "Unknown"}`;
                        return { label: labelText, value: variant };
                    });

                    // Auto-select highest quality
                    if (resolutions.value.length > 0) {
                        selectedResolution.value = resolutions.value[resolutions.value.length - 1];
                    }
                } else {
                    hasResolution.value = false;
                    resolutions.value = [];
                }

                mediaChecked.value = true;
            } else {
                errorMessage.value = result.message || "Unknown backend error.";
            }
        } catch (error: any) {
            console.error("Critical System Error:", error);
            errorMessage.value = typeof error === 'string' ? error : "System Error: " + JSON.stringify(error);
        } finally {
            loadingMedia.value = false;
        }
    };

    /**
     * Checks if the file exists on disk.
     * Returns: "Busy" | "Exists" | "Ok" | "Error"
     */
    const validateAndCheckFileStatus = async (): Promise<string> => {
        errorMessage.value = "";

        if (!videoUrl.value) { errorMessage.value = "Please enter a video URL."; return "Error"; }
        if (!saveFolder.value) { errorMessage.value = "Please select a download folder."; return "Error"; }
        if (!saveFilename.value.trim()) { errorMessage.value = "Please enter a filename."; return "Error"; }

        const finalFileName = `${saveFilename.value}.${fileExtension.value}`;

        try {
            const status = await invoke<string>('check_file_status', {
                folder: saveFolder.value,
                filename: finalFileName
            });
            return status;
        } catch (error: any) {
            errorMessage.value = "Failed to check file status: " + error;
            return "Error";
        }
    };

    const submitJob = async (overwrite: boolean) => {
        const httpHeaders: Record<string, string> = {};
        for (const h of headers.value) {
            if (h.name.trim() && h.value.trim()) {
                httpHeaders[h.name.trim()] = h.value.trim();
            }
        }

        let finalDownloadUrl = videoUrl.value;
        if (hasResolution.value && selectedResolution.value) {
            finalDownloadUrl = selectedResolution.value.value.uri;
        }

        const finalFileName = `${saveFilename.value}.${fileExtension.value}`;

        try {
            const result = await invoke<any>('create_job', {
                downloadUrl: finalDownloadUrl.trim(),
                saveFolder: saveFolder.value.trim(),
                fileName: finalFileName,
                httpHeaders: httpHeaders,
            });

            if (result.success) {
                return true;
            } else {
                errorMessage.value = result.message || "Unknown error starting download.";
                return false;
            }
        } catch (error: any) {
            errorMessage.value = "System error: " + error;
            return false;
        }
    };

    return {
        // State
        videoUrl, headers, headerOptions, saveFilename, fileExtension, saveFolder,
        mediaChecked, loadingMedia, loadingDownload, errorMessage, hasResolution, resolutions, selectedResolution, totalSegments,
        extensionOptions,
        
        // Actions
        resetForm,
        checkMediaMetadata,
        validateAndCheckFileStatus,
        submitJob,
        addHeader,
        removeHeader
    };
}