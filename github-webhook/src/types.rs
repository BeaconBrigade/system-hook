//! Types specified on github [docs](https://docs.github.com/en/webhooks-and-events/webhooks/webhook-events-and-payloads).
use serde::{Deserialize, Serialize};
use strum_macros::{EnumDiscriminants, EnumString};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GithubPayload {
    pub guid: Uuid,
    pub signature_sha1: Option<String>,
    pub signature_sha256: Option<String>,
    pub event: Event,
}

pub type Organization = Option<serde_json::Value>;
pub type Repository = Option<serde_json::Value>;
pub type Installation = Option<serde_json::Value>;
pub type Enterprise = Option<serde_json::Value>;
pub type Sender = serde_json::Value;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, EnumDiscriminants)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[strum_discriminants(derive(EnumString), strum(serialize_all = "snake_case"))]
pub enum Event {
    BranchProtectionRule {
        action: String,
        enterprise: Enterprise,
        repository: Repository,
        installation: Installation,
        organization: Organization,
        rule: BranchProtectionRule,
        sender: Sender,
        changes: Option<serde_json::Value>,
    },
    CheckRun {},
    CheckSuite {},
    CodeScanningAlert {},
    CommitComment {},
    Create {},
    Delete {},
    DependabotAlert {},
    DeployKey {},
    Deployment {},
    DeploymentStatus {},
    Discussion {},
    DiscussionComment {},
    Fork {},
    GithubAppAuthorization {},
    Gollum {},
    Installation {},
    InstallationRepositories {},
    InstallationTarget {},
    IssueComment {},
    Issues {},
    Label {},
    MarketplacePurchase {},
    Member {},
    Membership {},
    MergeGroup {},
    Meta {},
    Milestone {},
    OrgBlock {},
    Organization {},
    Package {},
    PageBuild {},
    Ping {
        hook: Option<Hook>,
        hook_id: Option<i64>,
        organization: Organization,
        repository: Repository,
        sender: Sender,
        zen: String,
    },
    ProjectCard {},
    Project {},
    ProjectColumn {},
    ProjectsV2 {},
    ProjectsV2Item {},
    Public {},
    PullRequest {},
    PullRequestReviewComments {},
    PullRequestReview {},
    PullRequestReviewThread {},
    Push {
        after: String,
        base_ref: Option<String>,
        before: String,
        commits: Vec<Commit>,
        compare: String,
        created: bool,
        deleted: bool,
        enterprise: Enterprise,
        forced: bool,
        // box for clippy
        head_commit: Box<HeadCommit>,
        installation: Installation,
        organization: Organization,
        pusher: Pusher,
        r#ref: String,
        repository: serde_json::Value,
        sender: Sender,
    },
    RegistryPackage {},
    Release {},
    Repository {},
    RepositoryDispatch {},
    RepositoryImport {},
    RepositoryVulnerabilityAlert {},
    SecretScanningAlert {},
    SecretScanningAlertLocation {},
    SecretAdvisory {},
    SecurityAndAnalysis {},
    Sponsorship {},
    Star {},
    Status {},
    TeamAdd {},
    Team {},
    Watch {},
    WorkflowDispatch {},
    WorkflowJob {},
    WorkflowRun {},
}

impl Default for Event {
    fn default() -> Self {
        Self::Ping {
            hook: Default::default(),
            hook_id: Default::default(),
            organization: Default::default(),
            repository: Default::default(),
            sender: Default::default(),
            zen: Default::default(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BranchProtectionRule {
    pub admin_enforced: bool,
    pub allow_deletions_enforcement_level: String,
    pub authorized_actor_names: Vec<String>,
    pub authorized_actors_only: bool,
    pub authorized_dismissal_actors_only: bool,
    pub create_protected: bool,
    pub created_at: String,
    pub dismiss_stale_reviews_on_push: bool,
    pub id: i64,
    pub ignore_approvals_from_contributors: bool,
    pub linear_history_requirement_enforcement_level: EnforcementLevel,
    pub merge_queue_enforcement_level: EnforcementLevel,
    pub name: String,
    pub pull_request_revies_enforcement_level: EnforcementLevel,
    pub repository_id: i64,
    pub require_code_owner_review: bool,
    pub required_approving_review_count: i64,
    pub required_conversation_resolution_level: EnforcementLevel,
    pub required_deployments_enforcement_level: EnforcementLevel,
    pub required_status_checks: Vec<String>,
    pub required_status_checks_enforcement_level: EnforcementLevel,
    pub signature_requirement_enforcement_level: EnforcementLevel,
    pub strict_required_status_checks_policy: bool,
    pub updated_at: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementLevel {
    #[default]
    Off,
    NonAdmins,
    Everyone,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Hook {
    active: bool,
    app_id: Option<i64>,
    config: Config,
    created_at: String,
    deliveries_url: Option<String>,
    events: Vec<String>,
    id: i64,
    last_response: Option<LastResponse>,
    name: Web,
    ping_url: Option<String>,
    test_url: Option<String>,
    r#type: String,
    updated_at: String,
    url: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Config {
    content_type: ContentType,
    insecure_ssl: NumberOrString,
    secret: String,
    url: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Json,
    #[default]
    Form,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum NumberOrString {
    Number(f64),
    String(String),
}

impl Default for NumberOrString {
    fn default() -> Self {
        Self::String("".into())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LastResponse {
    code: Option<i64>,
    status: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Web {
    #[default]
    Web,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Commit {
    added: Option<Vec<String>>,
    author: Author,
    committer: Committer,
    distinct: bool,
    id: String,
    message: String,
    modified: Option<Vec<String>>,
    removed: Option<Vec<String>>,
    timestamp: String,
    tree_id: String,
    url: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Author {
    date: Option<String>,
    email: String,
    name: String,
    username: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Committer {
    date: Option<String>,
    email: Option<String>,
    name: String,
    username: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct HeadCommit {
    added: Option<Vec<String>>,
    author: Author,
    committer: Committer,
    distinct: bool,
    id: String,
    message: String,
    modified: Option<Vec<String>>,
    removed: Option<Vec<String>>,
    timestamp: String,
    tree_id: String,
    url: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Pusher {
    date: Option<String>,
    email: Option<String>,
    name: String,
    username: Option<String>,
}
