export interface User {
  id: string;
  email: string;
  username: string;
  display_name: string;
  avatar_url: string | null;
  created_at: string;
}

export interface UserSummary {
  id: string;
  username: string;
  display_name: string;
  avatar_url: string | null;
}

export interface Project {
  id: string;
  key: string;
  name: string;
  description: string | null;
  owner_id: string;
  created_at: string;
}

export interface ProjectMember {
  user_id: string;
  username: string;
  display_name: string;
  avatar_url: string | null;
  role: string;
  joined_at: string;
}

export type TicketType = 'epic' | 'story' | 'task' | 'subtask' | 'bug';
export type TicketStatus = 'backlog' | 'todo' | 'in_progress' | 'in_review' | 'done';
export type TicketPriority = 'low' | 'medium' | 'high' | 'critical';

export interface Ticket {
  id: string;
  slug: string;
  ticket_number: number;
  project_id: string;
  ticket_type: TicketType;
  title: string;
  description: string | null;
  status: TicketStatus;
  priority: TicketPriority;
  assignee_id: string | null;
  reporter_id: string;
  parent_id: string | null;
  story_points: number | null;
  sprint_id: string | null;
  due_date: string | null;
  created_at: string;
  updated_at: string;
}

export interface Sprint {
  id: string;
  project_id: string;
  name: string;
  goal: string | null;
  start_date: string | null;
  end_date: string | null;
  status: 'planning' | 'active' | 'completed';
  created_at: string;
}

export interface Tag {
  id: string;
  project_id: string;
  name: string;
  color: string;
  created_at: string;
}

export interface Comment {
  id: string;
  ticket_id: string;
  author_id: string;
  author_username: string;
  author_display_name: string;
  author_avatar_url: string | null;
  parent_id: string | null;
  body: string;
  created_at: string;
  updated_at: string;
}

export interface TicketLink {
  id: string;
  source_id: string;
  target_id: string;
  target_slug: string;
  target_title: string;
  link_type: string;
  created_at: string;
}

export interface RepoLink {
  id: string;
  ticket_id: string;
  repo_url: string;
  branch_name: string | null;
  pr_url: string | null;
  created_at: string;
}

export interface ActivityEntry {
  id: string;
  ticket_id: string;
  actor_id: string;
  actor_username: string;
  actor_display_name: string;
  action: string;
  old_value: string | null;
  new_value: string | null;
  created_at: string;
}

export interface BoardColumn {
  status: string;
  tickets: Ticket[];
}

export interface Board {
  columns: BoardColumn[];
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  user: User;
}

export interface UserStub {
  id: string;
  username: string;
  display_name: string;
  avatar_url: string | null;
}

export interface TicketSummary {
  id: string;
  slug: string;
  title: string;
  ticket_type: TicketType;
  status: TicketStatus;
}

export interface TicketDetail extends Ticket {
  assignee: UserStub | null;
  reporter: UserStub;
  parent: TicketSummary | null;
  tags: Tag[];
  children: TicketSummary[];
}
