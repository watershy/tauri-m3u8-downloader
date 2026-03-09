import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { useToast } from "primevue/usetoast";

export function useFileCheck() {
    const toast = useToast();
    const missingFileIds = ref<Set<string>>(new Set());

    // Bulk checks existence for a list of completed jobs.
    // Updates the missingFileIds set.
    const checkMissingFiles = async (videos: any[]) => {
        const candidates = videos.filter(v =>
            v.status === 'CompletedSuccess' || v.statusLabel === 'Completed'
        );

        if (candidates.length === 0) return;

        const payload = candidates.map(job => {
            const separator = job.save_folder.includes('\\') ? '\\' : '/';
            const cleanPath = job.save_folder.replace(/[\\/]$/, '');
            const fullPath = `${cleanPath}${separator}${job.file_name}`;
            return [fullPath, job.final_file_size || null];
        });

        try {
            const results = await invoke<boolean[]>('check_files_exist', { files: payload });
            const newMissingSet = new Set(missingFileIds.value);
            
            results.forEach((exists, index) => {
                const jobId = candidates[index].id;
                if (!exists) {
                    newMissingSet.add(jobId);
                } else {
                    newMissingSet.delete(jobId);
                }
            });

            missingFileIds.value = newMissingSet;
        } catch (e) {
            console.warn("Failed to check file existence:", e);
        }
    };

    // Checks a specific job's file and opens the folder.
    const verifyAndOpenFolder = async (job: any) => {
        if (!job || !job.save_folder) return;

        const separator = job.save_folder.includes('\\') ? '\\' : '/';
        const cleanPath = job.save_folder.replace(/[\\/]$/, '');
        const fullPath = `${cleanPath}${separator}${job.file_name}`;

        try {
            // Check specific file
            const payload = [[fullPath, job.final_file_size || null]];
            const [exists] = await invoke<boolean[]>('check_files_exist', { files: payload });

            if (!exists) {
                if (!missingFileIds.value.has(job.id)) {
                    toast.add({
                        severity: 'warn',
                        summary: 'File Missing',
                        detail: 'The file appears to be moved or deleted.',
                        life: 3000
                    });
                }

                missingFileIds.value.add(job.id);
            } else {
                missingFileIds.value.delete(job.id);
            }

            // Open folder regardless
            await invoke('open_folder', { path: job.save_folder });

        } catch (e) {
            console.error("Failed to check/open folder:", e);
            // Fallback
            await invoke('open_folder', { path: job.save_folder });
        }
    };

    return {
        missingFileIds,
        checkMissingFiles,
        verifyAndOpenFolder
    };
}