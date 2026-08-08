import React, { useState, useEffect } from "react";

interface ButtonProps {
  label: string;
  onClick: () => void;
  variant?: "primary" | "secondary";
}

const Button: React.FC<ButtonProps> = ({ label, onClick, variant = "primary" }) => {
  const [clicked, setClicked] = useState(false);

  useEffect(() => {
    if (clicked) {
      const timer = setTimeout(() => setClicked(false), 200);
      return () => clearTimeout(timer);
    }
  }, [clicked]);

  return (
    <button
      className={`btn btn-${variant}`}
      onClick={() => {
        setClicked(true);
        onClick();
      }}
    >
      {label}
    </button>
  );
};

export default function App() {
  const [count, setCount] = useState(0);

  return (
    <div className="app">
      <h1>Counter App</h1>
      <p>Count: {count}</p>
      <Button label="Increment" onClick={() => setCount(count + 1)} />
      <Button label="Reset" onClick={() => setCount(0)} variant="secondary" />
      <img src="/logo.png" alt="Logo" />
    </div>
  );
}
