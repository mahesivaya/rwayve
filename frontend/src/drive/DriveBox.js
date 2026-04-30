import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { logger } from "../utils/logger";
import { useEffect, useState } from "react";
import "./drive.css";
import { useAuth } from "../auth/AuthContext";
export default function Drive() {
    const { user } = useAuth();
    const [files, setFiles] = useState([]);
    const [uploadedFiles, setUploadedFiles] = useState([]);
    const [loading, setLoading] = useState(false);
    const [uploading, setUploading] = useState(false);
    //
    // ✅ FETCH FILES
    //
    const fetchFiles = async () => {
        if (!user)
            return;
        setLoading(true);
        try {
            const res = await fetch(`/api/files?user_id=${user.id}`);
            if (!res.ok) {
                throw new Error("Failed to fetch files");
            }
            const data = await res.json();
            logger.log("🔥 API DATA:", data);
            logger.log("👤 USER ID:", user.id);
            setUploadedFiles(Array.isArray(data) ? data : []);
        }
        catch (err) {
            logger.error("❌ Fetch error:", err);
        }
        finally {
            setLoading(false);
        }
    };
    useEffect(() => {
        if (user?.id) {
            fetchFiles();
        }
    }, [user]);
    //
    // ✅ HANDLE FILE SELECT
    //
    const handleFiles = (selected) => {
        if (!selected)
            return;
        setFiles((prev) => [...prev, ...Array.from(selected)]);
    };
    //
    // ✅ DRAG DROP
    //
    const handleDrop = (e) => {
        e.preventDefault();
        handleFiles(e.dataTransfer.files);
    };
    //
    // ❌ REMOVE FILE FROM SELECTION (UX IMPROVEMENT)
    //
    const removeFile = (index) => {
        setFiles((prev) => prev.filter((_, i) => i !== index));
    };
    //
    // ✅ UPLOAD FILES
    //
    const uploadFiles = async () => {
        if (!files.length) {
            alert("No files selected");
            return;
        }
        if (!user) {
            alert("User not logged in");
            return;
        }
        setUploading(true);
        const formData = new FormData();
        // 🔥 IMPORTANT: user_id FIRST (backend reads it)
        formData.append("user_id", user.id.toString());
        files.forEach((f) => formData.append("files", f));
        try {
            const res = await fetch("/api/files/upload", {
                method: "POST",
                body: formData,
            });
            if (!res.ok)
                throw new Error("Upload failed");
            logger.log("✅ Upload success");
            setFiles([]);
            fetchFiles(); // refresh list
        }
        catch (err) {
            logger.error("❌ Upload error:", err);
            alert("Upload failed");
        }
        finally {
            setUploading(false);
        }
    };
    return (_jsxs("div", { className: "drive-container", children: [_jsxs("div", { className: "upload-section", children: [_jsxs("div", { className: "drive-header", children: [_jsx("h2", { children: "\uD83D\uDCC1 My Drive" }), _jsx("button", { className: "upload-btn", onClick: uploadFiles, disabled: uploading, children: uploading ? "Uploading..." : "Upload" })] }), _jsxs("div", { className: "drop-zone", onDragOver: (e) => e.preventDefault(), onDrop: handleDrop, children: [_jsx("p", { children: "Drag & Drop files here" }), _jsx("span", { children: "or" }), _jsxs("label", { className: "browse-btn", children: ["Browse Files", _jsx("input", { type: "file", multiple: true, onChange: (e) => handleFiles(e.target.files), hidden: true })] })] }), files.length > 0 && (_jsxs("div", { className: "selected-files", children: [_jsx("h4", { children: "\uD83D\uDCE4 Selected Files" }), _jsx("ul", { children: files.map((f, i) => (_jsxs("li", { children: [f.name, " (", (f.size / 1024).toFixed(2), " KB)", _jsx("button", { onClick: () => removeFile(i), children: "\u274C" })] }, i))) })] }))] }), _jsxs("div", { className: "files-section", children: [_jsx("h3", { children: "\uD83D\uDCC1 Uploaded Files" }), loading ? (_jsx("p", { children: "Loading..." })) : !uploadedFiles || uploadedFiles.length === 0 ? (_jsx("p", { children: "No files uploaded yet" })) : (_jsx("div", { className: "file-list", children: uploadedFiles.map((file) => (_jsxs("div", { className: "file-row", children: [_jsxs("div", { className: "file-left", children: [_jsx("span", { className: "file-icon", children: file.file_type === "png" || file.file_type === "jpg"
                                                ? "🖼️"
                                                : file.file_type === "pdf"
                                                    ? "📕"
                                                    : file.file_type === "zip"
                                                        ? "🗜️"
                                                        : "📄" }), _jsxs("div", { className: "file-main", children: [_jsx("div", { className: "file-name", children: file.name }), _jsxs("div", { className: "file-meta", children: [file.file_type, " \u2022 ", (file.size / 1024).toFixed(1), " KB"] })] })] }), _jsx("div", { className: "file-right", children: file.drive_url && (_jsx("a", { href: file.drive_url, target: "_blank", rel: "noreferrer", children: "Open" })) })] }, file.id))) }))] })] }));
}
