import { useEffect, useState } from "react";
import "./drive.css";
import { useAuth } from "../auth/AuthContext";

type UploadedFile = {
  id: number;
  name: string;
  file_type: string;
  size: number;
  drive_url?: string;
  created_at: string;
};

export default function Drive() {
  const { user } = useAuth();

  const [files, setFiles] = useState<File[]>([]);
  const [uploadedFiles, setUploadedFiles] = useState<UploadedFile[]>([]);

  const fetchFiles = async () => {
    if (!user) return;

    try {
      const res = await fetch(`/api/files?user_id=${user.id}`);
      const data = await res.json();
      setUploadedFiles(data);
    } catch (err) {
      console.error(err);
    }
  };

  useEffect(() => {
    fetchFiles();
  }, [user]);

  const handleFiles = (selected: FileList | null) => {
    if (!selected) return;
    setFiles((prev) => [...prev, ...Array.from(selected)]);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    handleFiles(e.dataTransfer.files);
  };

  const uploadFiles = async () => {
    if (!files.length) return alert("No files selected");

    const formData = new FormData();
    files.forEach((f) => formData.append("files", f));

    try {
      await fetch("/api/files/upload", {
        method: "POST",
        body: formData,
      });

      setFiles([]);
      fetchFiles();
    } catch {
      alert("Upload failed");
    }
  };

  return (
    <div className="drive-container">

      {/* 🔹 Upload Section */}
      <div className="upload-section">
        <div className="drive-header">
          <h2>📁 My Drive</h2>
          <button className="upload-btn" onClick={uploadFiles}>
            Upload
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
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>

      {/* 🔹 Files Section */}
      <div className="files-section">
        <h3>📁 Uploaded Files</h3>

        {uploadedFiles.length === 0 ? (
          <p>No files uploaded yet</p>
        ) : (
          <div className="file-list">
            {uploadedFiles.map((file) => (
              <div key={file.id} className="file-row">

                <div className="file-left">
                  <span className="file-icon">📄</span>

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