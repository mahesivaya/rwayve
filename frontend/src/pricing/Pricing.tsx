import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { listPlans, type Plan } from "../api/billing";
import "./pricing.css";

const BYTES_IN_GB = 1024 * 1024 * 1024;

function formatBytes(bytes: number): string {
  if (bytes >= BYTES_IN_GB) return `${(bytes / BYTES_IN_GB).toFixed(0)} GB`;
  return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
}

function priceLabel(plan: Plan): string {
  if (plan.amount_cents === 0) return "Free";
  return `${(plan.amount_cents / 100).toFixed(2)} ${plan.currency.toUpperCase()}`;
}

const INCLUDED_IN_EVERY_PLAN = [
  "Email with end-to-end encrypted chat",
  "Voice & video calling",
  "Drive file storage",
  "Notes, tasks, and scheduler",
  "AI Chat assistant",
];

const FAQ = [
  {
    q: "Can I change plans later?",
    a: "Yes — upgrade or downgrade any time from the Billing page. Changes are prorated by Stripe.",
  },
  {
    q: "What happens if I cancel?",
    a: "Your plan stays active until the end of the current billing period, then drops to the free tier.",
  },
  {
    q: "How does organization billing work?",
    a: "Organization plans are billed once for the whole workspace and cover every member. Only organization admins can change the plan.",
  },
];

function PlanCard({
  plan,
  onChoose,
}: {
  plan: Plan;
  onChoose: () => void;
}) {
  const isFree = plan.amount_cents === 0;
  const featureEntries = Object.entries(plan.features ?? {});
  return (
    <article className="pricing-plan">
      <h3>{plan.name}</h3>
      <p className="pricing-plan-price">
        {priceLabel(plan)}
        {!isFree && (
          <span className="pricing-plan-interval"> / {plan.billing_interval}</span>
        )}
      </p>
      {plan.description && (
        <p className="pricing-plan-desc">{plan.description}</p>
      )}
      <ul className="pricing-plan-features">
        <li>{formatBytes(plan.storage_limit_bytes)} storage</li>
        <li>
          {plan.seat_limit} {plan.seat_limit === 1 ? "seat" : "seats"}
        </li>
        <li>Billed {plan.billing_interval}ly</li>
        {featureEntries.map(([key, value]) => (
          <li key={key}>
            {key}: {String(value)}
          </li>
        ))}
      </ul>
      <button className="pricing-plan-cta" onClick={onChoose}>
        {isFree ? "Get started" : "Choose plan"}
      </button>
    </article>
  );
}

export default function Pricing() {
  const navigate = useNavigate();
  const [plans, setPlans] = useState<Plan[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    let alive = true;
    listPlans()
      .then((items) => {
        if (alive) setPlans(items);
      })
      .catch((err) => {
        if (alive) {
          setError(err instanceof Error ? err.message : "Failed to load plans");
        }
      })
      .finally(() => {
        if (alive) setLoading(false);
      });
    return () => {
      alive = false;
    };
  }, []);

  const personalPlans = useMemo(
    () => plans.filter((plan) => plan.audience === "personal"),
    [plans]
  );
  const organizationPlans = useMemo(
    () => plans.filter((plan) => plan.audience === "organization"),
    [plans]
  );

  return (
    <div className="pricing-page">
      <header className="pricing-header">
        <h1>Plans &amp; Pricing</h1>
        <p>
          One workspace for mail, chat, calls, files, notes, and AI — pick the
          plan that fits. Manage or switch any time from Billing.
        </p>
        <button
          className="pricing-billing-link"
          onClick={() => navigate("/billing")}
        >
          Go to Billing
        </button>
      </header>

      {error && <div className="pricing-error">{error}</div>}
      {loading ? (
        <div className="pricing-empty">Loading plans…</div>
      ) : (
        <>
          <section className="pricing-section">
            <h2>Personal</h2>
            <p className="pricing-section-sub">For individual accounts.</p>
            <div className="pricing-grid">
              {personalPlans.map((plan) => (
                <PlanCard
                  key={plan.id}
                  plan={plan}
                  onChoose={() => navigate("/billing")}
                />
              ))}
              {personalPlans.length === 0 && (
                <p className="pricing-empty">No personal plans available.</p>
              )}
            </div>
          </section>

          <section className="pricing-section">
            <h2>Organization</h2>
            <p className="pricing-section-sub">
              One subscription that covers every member of the organization.
            </p>
            <div className="pricing-grid">
              {organizationPlans.map((plan) => (
                <PlanCard
                  key={plan.id}
                  plan={plan}
                  onChoose={() => navigate("/billing")}
                />
              ))}
              {organizationPlans.length === 0 && (
                <p className="pricing-empty">
                  No organization plans available.
                </p>
              )}
            </div>
          </section>

          {plans.length > 0 && (
            <section className="pricing-section">
              <h2>Compare all plans</h2>
              <table className="pricing-table">
                <thead>
                  <tr>
                    <th>Plan</th>
                    <th>For</th>
                    <th>Price</th>
                    <th>Storage</th>
                    <th>Seats</th>
                  </tr>
                </thead>
                <tbody>
                  {plans.map((plan) => (
                    <tr key={plan.id}>
                      <td>{plan.name}</td>
                      <td className="pricing-cap">{plan.audience}</td>
                      <td>
                        {priceLabel(plan)}
                        {plan.amount_cents > 0
                          ? ` / ${plan.billing_interval}`
                          : ""}
                      </td>
                      <td>{formatBytes(plan.storage_limit_bytes)}</td>
                      <td>{plan.seat_limit}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </section>
          )}
        </>
      )}

      <section className="pricing-section">
        <h2>Every plan includes</h2>
        <ul className="pricing-included">
          {INCLUDED_IN_EVERY_PLAN.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      </section>

      <section className="pricing-section">
        <h2>Questions</h2>
        <div className="pricing-faq">
          {FAQ.map((item) => (
            <div key={item.q} className="pricing-faq-item">
              <strong>{item.q}</strong>
              <p>{item.a}</p>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
