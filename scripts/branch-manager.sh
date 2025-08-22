#!/bin/bash

# Branch management script for ProvChainOrg enhancement project

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[STATUS]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if we're in the right directory
check_project_directory() {
    if [[ ! -f "Cargo.toml" ]] || [[ ! -d "src" ]]; then
        print_error "Not in ProvChainOrg project directory"
        exit 1
    fi
}

# Function to check current git status
check_git_status() {
    print_status "Checking current git status..."
    git status --porcelain
}

# Function to create feature branches
create_feature_branches() {
    print_status "Creating feature branches..."
    
    # Check if branches already exist
    if git rev-parse --verify feature/owl2-enhancements >/dev/null 2>&1; then
        print_warning "Branch feature/owl2-enhancements already exists"
    else
        git checkout -b feature/owl2-enhancements main
        git commit --allow-empty -m "Initialize OWL2 enhancements branch
- Branch created for implementing advanced OWL2 features
- Will include: owl:hasKey, property chains, qualified cardinality"
        print_success "Created feature/owl2-enhancements branch"
    fi
    
    if git rev-parse --verify feature/generic-traceability >/dev/null 2>&1; then
        print_warning "Branch feature/generic-traceability already exists"
    else
        git checkout main
        git checkout -b feature/generic-traceability
        git commit --allow-empty -m "Initialize generic traceability branch
- Branch created for implementing domain-agnostic traceability
- Will include: generic core ontology, domain plugins, configuration-driven loading"
        print_success "Created feature/generic-traceability branch"
    fi
    
    if git rev-parse --verify feature/unified-owl2-generic >/dev/null 2>&1; then
        print_warning "Branch feature/unified-owl2-generic already exists"
    else
        git checkout main
        git checkout -b feature/unified-owl2-generic
        git commit --allow-empty -m "Initialize unified OWL2 + Generic Traceability branch
- Branch created for integrated implementation
- Will merge both OWL2 features and generic traceability"
        print_success "Created feature/unified-owl2-generic branch"
    fi
    
    # Return to main branch
    git checkout main
}

# Function to list all branches
list_branches() {
    print_status "Current branch structure:"
    git branch -a
}

# Function to switch to a specific branch
switch_branch() {
    local branch_name=$1
    if [[ -z "$branch_name" ]]; then
        print_error "Branch name required"
        return 1
    fi
    
    if git rev-parse --verify "$branch_name" >/dev/null 2>&1; then
        git checkout "$branch_name"
        print_success "Switched to branch $branch_name"
    else
        print_error "Branch $branch_name does not exist"
        return 1
    fi
}

# Function to merge branches
merge_branches() {
    local source_branch=$1
    local target_branch=$2
    
    if [[ -z "$source_branch" ]] || [[ -z "$target_branch" ]]; then
        print_error "Source and target branch names required"
        return 1
    fi
    
    # Switch to target branch
    switch_branch "$target_branch" || return 1
    
    # Pull latest changes
    git pull origin "$target_branch"
    
    # Merge source branch
    git merge "$source_branch"
    
    print_success "Merged $source_branch into $target_branch"
}

# Function to run tests
run_tests() {
    local branch_name=$1
    if [[ -n "$branch_name" ]]; then
        switch_branch "$branch_name" || return 1
    fi
    
    print_status "Running tests..."
    cargo test --lib
}

# Function to show help
show_help() {
    echo "ProvChainOrg Branch Management Script"
    echo ""
    echo "Usage: ./branch-manager.sh [command] [options]"
    echo ""
    echo "Commands:"
    echo "  status              Show current git status"
    echo "  create-branches     Create all feature branches"
    echo "  list-branches       List all branches"
    echo "  switch <branch>     Switch to specified branch"
    echo "  merge <source> <target>  Merge source branch into target branch"
    echo "  test [branch]       Run tests (optionally on specified branch)"
    echo "  help                Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./branch-manager.sh create-branches"
    echo "  ./branch-manager.sh switch feature/owl2-enhancements"
    echo "  ./branch-manager.sh merge feature/owl2-enhancements feature/unified-owl2-generic"
    echo "  ./branch-manager.sh test feature/generic-traceability"
}

# Main script logic
main() {
    check_project_directory
    
    case "$1" in
        status)
            check_git_status
            ;;
        create-branches)
            create_feature_branches
            ;;
        list-branches)
            list_branches
            ;;
        switch)
            switch_branch "$2"
            ;;
        merge)
            merge_branches "$2" "$3"
            ;;
        test)
            run_tests "$2"
            ;;
        help)
            show_help
            ;;
        *)
            show_help
            ;;
    esac
}

# Run main function with all arguments
main "$@"