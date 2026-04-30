#!/usr/bin/env python3
"""
Simple test script to verify Sphinx documentation builds correctly.
"""

import subprocess
import sys
import os

def test_sphinx_build():
    """Test if Sphinx can build the documentation without errors."""
    try:
        # Change to docs directory
        if not os.path.exists('conf.py'):
            print("❌ Error: Not in docs directory or conf.py not found")
            return False
        
        # Test basic HTML build
        print("🔍 Testing Sphinx HTML build...")
        result = subprocess.run([
            'sphinx-build', '-b', 'html', '-W', '-E', '.', '_build/test'
        ], capture_output=True, text=True, timeout=60)
        
        if result.returncode == 0:
            print("✅ Sphinx HTML build successful!")
            return True
        else:
            print("❌ Sphinx build failed:")
            print("STDOUT:", result.stdout)
            print("STDERR:", result.stderr)
            return False
            
    except subprocess.TimeoutExpired:
        print("❌ Build timed out after 60 seconds")
        return False
    except Exception as e:
        print(f"❌ Error during build: {e}")
        return False

def main():
    """Main test function."""
    print("🚀 Testing ProvChainOrg Documentation Build")
    print("=" * 50)
    
    success = test_sphinx_build()
    
    if success:
        print("\n🎉 All tests passed! Documentation builds successfully.")
        sys.exit(0)
    else:
        print("\n💥 Tests failed. Please check the configuration.")
        sys.exit(1)

if __name__ == "__main__":
    main()
