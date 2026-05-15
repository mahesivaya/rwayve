import { Navigate, useNavigate, useParams } from "react-router-dom";
import { SERVICE_BY_SLUG, type ServiceSlug } from "./serviceData";
import "./servicePage.css";

export default function ServicePage() {
  const navigate = useNavigate();
  const { slug } = useParams();
  const service = slug ? SERVICE_BY_SLUG[slug as ServiceSlug] : null;

  if (!service) {
    return <Navigate to="/" replace />;
  }

  return (
    <div className="service-page">
      <header className="service-page-nav">
        <button className="service-page-brand" onClick={() => navigate("/")}>
          Wayve
        </button>
        <nav aria-label="Service navigation">
          <button onClick={() => navigate("/")}>Services</button>
          <button onClick={() => navigate("/business")}>Business</button>
          <button onClick={() => navigate("/#pricing")}>Pricing</button>
          <button onClick={() => navigate("/login")}>Login</button>
          <button className="service-page-register" onClick={() => navigate("/register")}>
            Register
          </button>
        </nav>
      </header>

      <main className="service-page-main">
        <section className="service-page-hero">
          <div className={`service-page-icon ${service.accent}`}>{service.icon}</div>
          <p className="service-page-eyebrow">{service.eyebrow}</p>
          <h1>{service.name}</h1>
          <p className="service-page-summary">{service.summary}</p>
          <p className="service-page-description">{service.description}</p>
          <div className="service-page-actions">
            <button onClick={() => navigate(service.appPath)}>Open {service.name}</button>
            <button onClick={() => navigate("/register")}>Create account</button>
          </div>
        </section>

        <section className="service-page-section">
          <div>
            <p className="service-page-section-label">Features</p>
            <h2>What you can do</h2>
          </div>
          <div className="service-feature-grid">
            {service.features.map((feature) => (
              <article key={feature}>
                <span className="feature-check">✓</span>
                <p>{feature}</p>
              </article>
            ))}
          </div>
        </section>

        <section className="service-page-section service-page-usecases">
          <div>
            <p className="service-page-section-label">Use cases</p>
            <h2>Built for daily work</h2>
          </div>
          <div className="service-usecase-list">
            {service.useCases.map((useCase) => (
              <p key={useCase}>{useCase}</p>
            ))}
          </div>
        </section>
      </main>
    </div>
  );
}
