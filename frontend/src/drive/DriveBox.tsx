import { logger } from "../utils/logger";
import { useEffect, useState } from "react";
import "./drive.css";
import {
  getDriveFiles,
  uploadDriveFiles,
  type UploadedFile,
} from "../api/drive";
import { useAuth } from "../auth/AuthContext";
import { useGlobalSearch } from "../search/SearchContext";

export default function Drive() {
  const { user } = useAuth();
  const { normalizedSearchQuery } = useGlobalSearch();

  const [files, setFiles] = useState<File[]>([]);
  const [uploadedFiles, setUploadedFiles] = useState<UploadedFile[]>([]);
  const [loading, setLoading] = useState(false);
  const [uploading, setUploading] = useState(false);

  //
  // ✅ FETCH FILES
  //
  const fetchFiles = async () => {
    if (!user) return;

    setLoading(true);

    try {
      const data = await getDriveFiles(user.id);

      logger.log("🔥 API DATA:", data);
      logger.log("👤 USER ID:", user.id);

      setUploadedFiles(Array.isArray(data) ? data : []);
    } catch (err) {
      logger.error("❌ Fetch error:", err);
    } finally {
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
  const handleFiles = (selected: FileList | null) => {
    if (!selected) return;
    setFiles((prev) => [...prev, ...Array.from(selected)]);
  };

  //
  // ✅ DRAG DROP
  //
  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    handleFiles(e.dataTransfer.files);
  };

  //
  // ❌ REMOVE FILE FROM SELECTION (UX IMPROVEMENT)
  //
  const removeFile = (index: number) => {
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

    try {
      await uploadDriveFiles(user.id, files);

      logger.log("✅ Upload success");

      setFiles([]);
      fetchFiles(); // refresh list
    } catch (err) {
      logger.error("❌ Upload error:", err);
      alert("Upload failed");
    } finally {
      setUploading(false);
    }
  };

  const visibleUploadedFiles = normalizedSearchQuery
    ? uploadedFiles.filter((file) =>
        [file.name, file.file_type, file.size.toString(), file.created_at]
          .join(" ")
          .toLowerCase()
          .includes(normalizedSearchQuery)
      )
    : uploadedFiles;

  return (
    <div className="drive-container">

      {/* 🔹 Upload Section */}
      <div className="upload-section">
        <div className="drive-header">
          <h2>📁 My Drive</h2>

          <button
            className="upload-btn"
            onClick={uploadFiles}
            disabled={uploading}
          >
            {uploading ? "Uploading..." : "Upload"}
          </button>
        </div>

        <div
          className="drop-zone"
          onDragOver={(e) => e.preventDefault()}
          onDrop={handleDrop}
        >
          <p>Drag & Drop files here</p>
          <span>or</span>

          <label className="browse-btn">
            Browse Files
            <input
              type="file"
              multiple
              onChange={(e) => handleFiles(e.target.files)}
              hidden
            />
          </label>
        </div>

        {files.length > 0 && (
          <div className="selected-files">
            <h4>📤 Selected Files</h4>
            <ul>
              {files.map((f, i) => (
                <li key={i}>
                  {f.name} ({(f.size / 1024).toFixed(2)} KB)
                  <button onClick={() => removeFile(i)}>❌</button>
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>

      {/* 🔹 Files Section */}
      <div className="files-section">
        <h3>📁 Uploaded Files</h3>

        {loading ? (
          <p>Loading...</p>
        ) : !visibleUploadedFiles || visibleUploadedFiles.length === 0 ? (
          <p>{normalizedSearchQuery ? "No files match your search" : "No files uploaded yet"}</p>
        ) : (
          <div className="file-list">
            {visibleUploadedFiles.map((file) => (
              <div key={file.id} className="file-row">

                <div className="file-left">
                  <span className="file-icon">
                    {file.file_type === "png" || file.file_type === "jpg"
                      ? "🖼️"
                      : file.file_type === "pdf"
                      ? "📕"
                      : file.file_type === "zip"
                      ? "🗜️"
                      : "📄"}
                  </span>

                  <div className="file-main">
                    <div className="file-name">{file.name}</div>
                    <div className="file-meta">
                      {file.file_type} • {(file.size / 1024).toFixed(1)} KB
                    </div>
                  </div>
                </div>

                <div className="file-right">
                  {file.drive_url && (
                    <a href={file.drive_url} target="_blank" rel="noreferrer">
                      Open
                    </a>
                  )}
                </div>

              </div>
            ))}
          </div>
        )}
      </div>

    </div>
  );
}
