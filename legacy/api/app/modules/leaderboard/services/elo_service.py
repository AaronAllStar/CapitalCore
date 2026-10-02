"""ELO rating system for tournament rankings."""
K_FACTOR = 32
DEFAULT_RATING = 1200


def expected_score(rating_a: float, rating_b: float) -> float:
    return 1 / (1 + 10 ** ((rating_b - rating_a) / 400))


def update_ratings(rating_a: float, rating_b: float, score_a: float) -> tuple[float, float]:
    """Update two ratings. score_a: 1.0 = A wins, 0.0 = B wins, 0.5 = draw."""
    exp_a = expected_score(rating_a, rating_b)
    exp_b = 1 - exp_a
    score_b = 1 - score_a

    new_a = rating_a + K_FACTOR * (score_a - exp_a)
    new_b = rating_b + K_FACTOR * (score_b - exp_b)

    return round(new_a), round(new_b)


def calculate_tournament_elo_changes(
    rankings: list[dict],
) -> list[dict]:
    """
    Calculate ELO changes for a tournament.
    rankings: [{user_id, rating, rank}, ...]
    Returns same list with 'rating_change' and 'new_rating' added.
    """
    if len(rankings) < 2:
        return [{**r, "rating_change": 0, "new_rating": r["rating"]} for r in rankings]

    # Sort by rank
    sorted_r = sorted(rankings, key=lambda x: x["rank"])

    changes = {r["user_id"]: 0 for r in sorted_r}

    # Each participant "plays" against participants close to them
    for i in range(len(sorted_r)):
        for j in range(i + 1, min(i + 4, len(sorted_r))):
            a = sorted_r[i]
            b = sorted_r[j]

            # Higher rank (lower number) = win
            score_a = 1.0 if a["rank"] < b["rank"] else (0.0 if a["rank"] > b["rank"] else 0.5)

            exp_a = expected_score(a["rating"], b["rating"])
            exp_b = 1 - exp_a
            score_b = 1 - score_a

            changes[a["user_id"]] += K_FACTOR * (score_a - exp_a)
            changes[b["user_id"]] += K_FACTOR * (score_b - exp_b)

    result = []
    for r in sorted_r:
        change = round(changes[r["user_id"]])
        result.append({
            **r,
            "rating_change": change,
            "new_rating": max(100, r["rating"] + change),
        })

    return result
