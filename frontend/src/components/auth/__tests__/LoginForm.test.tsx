import React from "react";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { AuthProvider } from "../../../contexts/AuthContext";
import LoginForm from "../LoginForm";
import { customRender } from "../../../test-utils/test-utils";

// Mock the AuthContext
const mockLogin = jest.fn();
jest.mock("../../../contexts/AuthContext", () => ({
  ...jest.requireActual("../../../contexts/AuthContext"),
  useAuth: () => ({
    login: mockLogin,
  }),
}));

// Mock onLoginSuccess callback
const mockOnLoginSuccess = jest.fn();

const renderLoginForm = (props = {}) => {
  const defaultProps = {
    onLoginSuccess: mockOnLoginSuccess,
    ...props,
  };

  return customRender(
    <AuthProvider>
      <LoginForm {...defaultProps} />
    </AuthProvider>,
  );
};

describe("LoginForm Component", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockLogin.mockClear();
    mockOnLoginSuccess.mockClear();
  });

  describe("Component Rendering", () => {
    test("should render login form correctly", () => {
      renderLoginForm();

      expect(screen.getByText("ProvChain-Org Login")).toBeInTheDocument();
      expect(
        screen.getByText("Secure access to supply chain traceability system"),
      ).toBeInTheDocument();

      // Check form inputs
      expect(screen.getByLabelText(/username/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
      expect(
        screen.getByRole("button", { name: /sign in/i }),
      ).toBeInTheDocument();

      // Check development text
      expect(screen.getByText(/For development:/i)).toBeInTheDocument();
    });

    test("should have proper form structure", () => {
      renderLoginForm();

      const form = document.querySelector("form");
      expect(form).toBeInTheDocument();

      const usernameInput = screen.getByLabelText(/username/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole("button", { name: /sign in/i });

      expect(usernameInput).toHaveAttribute("type", "text");
      expect(usernameInput).toHaveAttribute("autoComplete", "username");
      expect(usernameInput).toHaveAttribute("required");

      expect(passwordInput).toHaveAttribute("type", "password");
      expect(passwordInput).toHaveAttribute("autoComplete", "current-password");
      expect(passwordInput).toHaveAttribute("required");

      expect(submitButton).toHaveAttribute("type", "submit");
    });
  });

  describe("Form Interaction", () => {
    test("should allow typing in username and password fields", async () => {
      const user = userEvent.setup();
      renderLoginForm();

      const usernameInput = screen.getByLabelText(/username/i);
      const passwordInput = screen.getByLabelText(/password/i);

      await user.type(usernameInput, "testuser");
      await user.type(passwordInput, "testpassword");

      expect(usernameInput).toHaveValue("testuser");
      expect(passwordInput).toHaveValue("testpassword");
    });

    test("should validate empty fields on submit", async () => {
      const user = userEvent.setup();
      renderLoginForm();

      const submitButton = screen.getByRole("button", { name: /sign in/i });
      await user.click(submitButton);

      expect(
        screen.getByText(/Please enter both username and password/i),
      ).toBeInTheDocument();
      expect(mockLogin).not.toHaveBeenCalled();
    });

    test("should validate empty username field", async () => {
      const user = userEvent.setup();
      renderLoginForm();

      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole("button", { name: /sign in/i });

      await user.type(passwordInput, "password123");
      await user.click(submitButton);

      expect(
        screen.getByText(/Please enter both username and password/i),
      ).toBeInTheDocument();
      expect(mockLogin).not.toHaveBeenCalled();
    });

    test("should validate empty password field", async () => {
      const user = userEvent.setup();
      renderLoginForm();

      const usernameInput = screen.getByLabelText(/username/i);
      const submitButton = screen.getByRole("button", { name: /sign in/i });

      await user.type(usernameInput, "testuser");
      await user.click(submitButton);

      await waitFor(() => {
        expect(
          screen.getByText(/Please enter both username and password/i),
        ).toBeInTheDocument();
      });
      expect(mockLogin).not.toHaveBeenCalled();
    });

    test("should trim username whitespace", async () => {
      const user = userEvent.setup();
      mockLogin.mockResolvedValue({ success: true });

      renderLoginForm();

      const usernameInput = screen.getByLabelText(/username/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole("button", { name: /sign in/i });

      await user.type(usernameInput, "  testuser  ");
      await user.type(passwordInput, "testpassword");
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockLogin).toHaveBeenCalledWith("testuser", "testpassword");
      });
    });
  });

  describe("Login Submission", () => {
    test("should call login function with correct credentials", async () => {
      const user = userEvent.setup();
      mockLogin.mockResolvedValue({ success: true });

      renderLoginForm();

      const usernameInput = screen.getByLabelText(/username/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole("button", { name: /sign in/i });

      await user.type(usernameInput, "testuser");
      await user.type(passwordInput, "testpassword");
      await user.click(submitButton);

      expect(mockLogin).toHaveBeenCalledWith("testuser", "testpassword");
    });

    test("should call onLoginSuccess on successful login", async () => {
      const user = userEvent.setup();
      mockLogin.mockResolvedValue({ success: true });

      renderLoginForm();

      const usernameInput = screen.getByLabelText(/username/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole("button", { name: /sign in/i });

      await user.type(usernameInput, "testuser");
      await user.type(passwordInput, "testpassword");
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockOnLoginSuccess).toHaveBeenCalled();
      });
    });

    test("should show error message on failed login", async () => {
      const user = userEvent.setup();
      mockLogin.mockResolvedValue({
        success: false,
        error: "Invalid credentials",
      });

      renderLoginForm();

      const usernameInput = screen.getByLabelText(/username/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole("button", { name: /sign in/i });

      await user.type(usernameInput, "wronguser");
      await user.type(passwordInput, "wrongpass");
      await user.click(submitButton);

      await waitFor(() => {
        expect(screen.getByText("Invalid credentials")).toBeInTheDocument();
      });

      expect(mockOnLoginSuccess).not.toHaveBeenCalled();
    });

    test("should show default error message when login fails without specific error", async () => {
      const user = userEvent.setup();
      mockLogin.mockResolvedValue({ success: false });

      renderLoginForm();

      const usernameInput = screen.getByLabelText(/username/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole("button", { name: /sign in/i });

      await user.type(usernameInput, "testuser");
      await user.type(passwordInput, "testpassword");
      await user.click(submitButton);

      await waitFor(() => {
        expect(screen.getByText("Login failed")).toBeInTheDocument();
      });
    });

    test("should not call onLoginSuccess on failed login", async () => {
      const user = userEvent.setup();
      mockLogin.mockResolvedValue({
        success: false,
        error: "Invalid credentials",
      });

      renderLoginForm();

      const usernameInput = screen.getByLabelText(/username/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole("button", { name: /sign in/i });

      await user.type(usernameInput, "testuser");
      await user.type(passwordInput, "wrongpassword");
      await user.click(submitButton);

      await waitFor(() => {
        expect(screen.getByText("Invalid credentials")).toBeInTheDocument();
      });

      expect(mockOnLoginSuccess).not.toHaveBeenCalled();
    });
  });

  describe("Loading States", () => {
    test("should show loading state during login submission", async () => {
      const user = userEvent.setup();
      let resolveLogin: (value: { success: boolean; error?: string }) => void;
      const loginPromise = new Promise((resolve) => {
        resolveLogin = resolve;
      });

      mockLogin.mockReturnValue(loginPromise);

      renderLoginForm();

      const usernameInput = screen.getByLabelText(/username/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole("button", { name: /sign in/i });

      await user.type(usernameInput, "testuser");
      await user.type(passwordInput, "testpassword");

      // Submit the form
      await user.click(submitButton);

      // Check loading state
      expect(screen.getByText(/Signing in\.\.\./i)).toBeInTheDocument();
      expect(
        screen.getByRole("button", { name: /signing in/i }),
      ).toBeDisabled();
      expect(screen.getByLabelText(/username/i)).toBeDisabled();
      expect(screen.getByLabelText(/password/i)).toBeDisabled();

      // Resolve the login promise
      resolveLogin({ success: true });

      await waitFor(() => {
        expect(screen.queryByText(/Signing in\.\.\./i)).not.toBeInTheDocument();
      });
    });
  });

  describe("Error Handling", () => {
    test("should clear error message when user starts typing", async () => {
      const user = userEvent.setup();
      mockLogin
        .mockResolvedValueOnce({
          success: false,
          error: "Invalid credentials",
        })
        .mockResolvedValueOnce({ success: true });

      renderLoginForm();

      const usernameInput = screen.getByLabelText(/username/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole("button", { name: /sign in/i });

      // First, trigger an error
      await user.type(usernameInput, "wronguser");
      await user.type(passwordInput, "wrongpass");
      await user.click(submitButton);

      await waitFor(() => {
        expect(screen.getByText("Invalid credentials")).toBeInTheDocument();
      });

      // Clear the form and try again
      await user.clear(usernameInput);
      await user.clear(passwordInput);
      await user.type(usernameInput, "correctuser");
      await user.type(passwordInput, "correctpass");
      await user.click(submitButton);

      // Error should be cleared
      await waitFor(() => {
        expect(
          screen.queryByText("Invalid credentials"),
        ).not.toBeInTheDocument();
      });
    });
  });

  describe("Accessibility", () => {
    test("should have proper accessibility attributes", () => {
      renderLoginForm();

      // Check for proper labels
      expect(screen.getByLabelText(/username/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/password/i)).toBeInTheDocument();

      // Check for proper button text
      const submitButton = screen.getByRole("button", { name: /sign in/i });
      expect(submitButton).toBeInTheDocument();

      // Check for form submission capability
      const form = document.querySelector("form");
      expect(form).toBeInTheDocument();
    });

    test("should support keyboard navigation", async () => {
      const user = userEvent.setup();
      renderLoginForm();

      const usernameInput = screen.getByLabelText(/username/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole("button", { name: /sign in/i });

      // Tab through form elements
      await user.tab(); // Should focus username input
      expect(usernameInput).toHaveFocus();

      await user.tab(); // Should focus password input
      expect(passwordInput).toHaveFocus();

      await user.tab(); // Should focus submit button
      expect(submitButton).toHaveFocus();
    });

    test("should support form submission with Enter key", async () => {
      const user = userEvent.setup();
      mockLogin.mockResolvedValue({ success: true });

      renderLoginForm();

      const passwordInput = screen.getByLabelText(/password/i);
      const usernameInput = screen.getByLabelText(/username/i);

      await user.type(usernameInput, "testuser");
      await user.type(passwordInput, "testpassword{enter}");

      await waitFor(() => {
        expect(mockLogin).toHaveBeenCalledWith("testuser", "testpassword");
      });
    });
  });
});
