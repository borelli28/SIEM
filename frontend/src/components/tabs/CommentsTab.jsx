import React, { useState, useEffect } from 'react';
import { getCsrfToken } from '../../services/csrfService';
import '../../styles/CaseTabs.css';

const CommentsTab = ({ caseId, formId, showAlert }) => {
    const [comments, setComments] = useState([]);
    const [showCommentForm, setShowCommentForm] = useState(false);

    useEffect(() => {
        fetchComments();
    }, []);

    const fetchComments = async () => {
        try {
            const response = await fetch(`http://localhost:4200/backend/case/${caseId}/comments`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (!response.ok) {
                throw new Error('Failed to fetch comments');
            }

            const data = await response.json();
            setComments(data);
        } catch (err) {
            showAlert('Failed to fetch comments');
        }
    };

    const handleAddComment = async (e) => {
        e.preventDefault();
        const commentText = e.target.comment.value;

        try {
            const csrfToken = await getCsrfToken(formId);
            const response = await fetch(`http://localhost:4200/backend/case/${caseId}/comment`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include',
                body: JSON.stringify(commentText)
            });

            if (!response.ok) {
                throw new Error('Failed to add comment');
            }

            showAlert('Comment added successfully', 'success');
            e.target.reset();
            setShowCommentForm(false);
            await fetchComments();
        } catch (err) {
            showAlert('Failed to add comment');
        }
    };

    const handleDeleteComment = async (commentId) => {
        if (!window.confirm('Are you sure you want to delete this comment?')) {
            return;
        }

        try {
            const csrfToken = await getCsrfToken(formId);
            const response = await fetch(`http://localhost:4200/backend/case/comment/${commentId}`, {
                method: 'DELETE',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (!response.ok) {
                throw new Error('Failed to delete comment');
            }

            showAlert('Comment deleted successfully', 'success');
            await fetchComments();
        } catch (err) {
            showAlert('Failed to delete comment');
        }
    };

    return (
        <div className="tab-container">
            <div className="comments-section">
                <div className="add-comment-container">
                    <form onSubmit={handleAddComment}>
                        <textarea
                            id="comment-text"
                            name="comment"
                            placeholder="Enter your comment..."
                            required
                        ></textarea>
                        <button type="submit" className="primary-btn">Add Comment</button>
                    </form>
                </div>

                <div className="comments-list">
                    {comments.map(comment => (
                        <div key={comment.id} className="comment">
                            <div className="comment-header">
                                <div className="comment-content">
                                    {comment.comment}
                                </div>
                                <button
                                    className="delete-comment-btn"
                                    onClick={() => handleDeleteComment(comment.id)}
                                >
                                    Delete
                                </button>
                            </div>
                            <div className="comment-metadata">
                                {new Date(comment.created_at).toLocaleString()}
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
};

export default CommentsTab;