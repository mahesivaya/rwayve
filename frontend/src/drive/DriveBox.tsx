import { useState } from "react";
import "./drive.css";

export default function Drive() {
  const [files, setFiles] = useState<File[]>([]);

  const handleFiles = (selected: FileList | null) => {
    if (!selected) return;
    const arr = Array.from(selected);
    setFiles(prev => [...prev, ...arr]);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    handleFiles(e.dataTransfer.files);
  };

  const uploadFiles = async () => {
    if (!files.length) return alert("No files selected");

    const formData = new FormData();
    files.forEach(f => formData.append("files", f));

    try {
      const res = await fetch("/api/files/upload", {
        method: "POST",
        body: formData,
      });

      if (!res.ok) throw new Error("Upload failed");

      alert("Upload successful 🚀");
      setFiles([]);
    } catch (err) {
      console.error(err);
      alert("Upload error");
    }
  };

  return (
    <div className="drive-container">

      {/* HEADER */}
      <div className="drive-header">
        <h2>📁 My Drive</h2>
        <button className="upload-btn" onClick={uploadFiles}>
          Upload
        </button>
      </div>

      {/* DROP ZONE */}
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

      {/* FILE LIST */}
      <div className="file-list">
        {files.map((file, i) => (
          <div key={i} className="file-item">
            <div className="file-info">
              <span className="file-icon">📄</span>
              <div>
                <p className="file-name">{file.name}</p>
                <p className="file-size">
                  {(file.size / 1024).toFixed(2)} KB
                </p>
              </div>
            </div>
          </div>
        ))}
      </div>

    </div>
  );
}